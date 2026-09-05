use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use rangular_aot::HostCell;
use rangular_expr::{parse, Expr, Host, PipeRegistry, Value};
use rangular_host::{EventPayload, HostError, LoopScope};

type ValueMap = HashMap<String, Value>;
type CallLog = Vec<(String, Vec<Value>)>;

struct MapHost {
    values: Rc<RefCell<ValueMap>>,
    calls: Rc<RefCell<CallLog>>,
}

impl MapHost {
    fn new() -> Self {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Host for MapHost {
    fn get(&self, name: &str) -> Option<Value> {
        self.values.borrow().get(name).cloned()
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        self.values.borrow_mut().insert(name.to_owned(), value);
        Ok(())
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        self.calls
            .borrow_mut()
            .push((name.to_owned(), args.to_vec()));
        Ok(Value::Str(format!("called:{name}")))
    }
}

fn expr(input: &str) -> Expr {
    parse(input).expr.expect("expr")
}

#[test]
fn host_cell_eval_and_display_paths() {
    let host = MapHost::new();
    host.values
        .borrow_mut()
        .insert("flag".into(), Value::Bool(true));
    host.values
        .borrow_mut()
        .insert("label".into(), Value::Str("Hi".into()));
    host.values.borrow_mut().insert("n".into(), Value::Num(2.0));
    host.values.borrow_mut().insert(
        "items".into(),
        Value::List(vec![Value::Str("a".into()), Value::Num(1.0)]),
    );
    host.values.borrow_mut().insert(
        "evt".into(),
        Value::from(EventPayload::Click { x: 1, y: 2 }),
    );

    let cell = HostCell::new(host);
    let cloned = cell.clone();
    assert_eq!(
        cell.prop_str(&expr("label")),
        cloned.prop_str(&expr("label"))
    );
    assert!(cell.eval_truthy(&expr("flag")));
    assert!(cloned.eval_bool(&expr("flag")));
    assert!(cloned.eval_bool(&expr("label")));
    assert_eq!(cloned.eval_value(&expr("n")), Value::Num(2.0));
    assert_eq!(cloned.prop_str(&expr("n")), "2");
    assert_eq!(cloned.prop_str(&expr("flag")), "true");
    assert_eq!(cloned.prop_str(&expr("items")), "a,1");
    assert_eq!(cloned.prop_str(&expr("evt")), "event:click:1,2");
    assert_eq!(cloned.eval_list(&expr("items")), vec!["a".to_owned()]);
    assert_eq!(cell.eval_list(&expr("label")), Vec::<String>::new());
}

#[test]
fn host_cell_scoped_pipe_item_and_implicits() {
    let host = MapHost::new();
    let cell = HostCell::with_pipes(host, Arc::new(PipeRegistry::with_builtins()));
    let scope = LoopScope {
        item_name: Some("item"),
        item_val: Some("row"),
        index: Some(0),
        count: Some(2),
    };
    assert_eq!(
        cell.eval_scoped(&expr("item"), scope),
        Value::Str("row".into())
    );
    assert_eq!(cell.eval_scoped(&expr("$index"), scope), Value::Num(0.0));
    assert!(cell.eval_truthy_scoped(&expr("$first"), scope));
    assert!(cell.eval_bool_scoped(&expr("$even"), scope));
    assert_eq!(
        cell.eval_scoped(&expr("item | uppercase"), scope),
        Value::Str("ROW".into())
    );
    assert_eq!(cell.prop_str_scoped(&expr("item"), scope), "row".to_owned());
    let called = cell.eval_scoped(&expr("onTap(item)"), scope);
    assert_eq!(called, Value::Str("called:onTap".into()));
}

#[test]
fn host_cell_events_and_value_display_variants() {
    let host = MapHost::new();
    let calls = Rc::clone(&host.calls);
    let cell = HostCell::new(host);

    cell.emit_call("ping", &[Value::Num(1.0)]);
    assert_eq!(calls.borrow().len(), 1);

    let handler = expr("onInput($event)");
    cell.emit_event_call("onInput", &handler);
    cell.emit_dom_event_call("onInput", &handler, "click", "9,8".into());
    cell.emit_dom_event_call("onInput", &handler, "error", String::new());
    cell.emit_dom_event_call("onInput", &handler, "load", String::new());
    cell.emit_dom_event_call("onInput", &handler, "focus", "x".into());

    assert!(cell
        .eval_value(&Expr::Ident("$event".into()))
        .as_event()
        .is_some());

    let host2 = MapHost::new();
    host2.values.borrow_mut().insert(
        "inp".into(),
        Value::from(EventPayload::Input { value: "t".into() }),
    );
    host2
        .values
        .borrow_mut()
        .insert("err".into(), Value::from(EventPayload::Error));
    host2
        .values
        .borrow_mut()
        .insert("load".into(), Value::from(EventPayload::Load));
    host2.values.borrow_mut().insert(
        "custom".into(),
        Value::from(EventPayload::Custom(Box::new(Value::Str("c".into())))),
    );
    host2.values.borrow_mut().insert("unit".into(), Value::Unit);
    let cell2 = HostCell::new(host2);
    assert_eq!(cell2.prop_str(&expr("inp")), "event:input:t");
    assert_eq!(cell2.prop_str(&expr("err")), "event:error");
    assert_eq!(cell2.prop_str(&expr("load")), "event:load");
    assert_eq!(cell2.prop_str(&expr("custom")), "event:custom:c");
    assert_eq!(cell2.prop_str(&expr("unit")), "");
}

#[test]
fn host_cell_scoped_bool_none_item_and_event_helpers() {
    let host = MapHost::new();
    host.values
        .borrow_mut()
        .insert("label".into(), Value::Str("Hi".into()));
    let cell = HostCell::new(host);
    let scope = LoopScope {
        item_name: Some("item"),
        item_val: None,
        index: Some(1),
        count: Some(2),
    };
    assert!(cell.eval_bool_scoped(&expr("label"), scope));
    assert!(!cell.eval_truthy_scoped(&expr("missing"), scope));

    let scope_item = LoopScope {
        item_name: Some("item"),
        item_val: Some("row"),
        index: Some(0),
        count: Some(1),
    };
    cell.emit_event_call_scoped("onTap", &expr("onTap(item)"), scope_item);
    cell.emit_event_call("noop", &Expr::Ident("noCall".into()));

    let seed = MapHost::new();
    let values = Rc::clone(&seed.values);
    let cell2 = HostCell::new(seed);
    let write = rangular_parser::banana_write_expr(&Expr::Ident("seed".into()));
    cell2.emit_dom_event_call("$bananaSet", &write, "click", "1,2".into());
    assert!(values
        .borrow()
        .get("seed")
        .and_then(Value::as_event)
        .is_some());
}

#[test]
fn host_cell_compile_covers_bool_ne_and_add_literals() {
    use rangular_aot::{compile, compile_tokens, tokens_to_rust_source};

    let bool_lit = compile("@if (true) { <span>y</span> }", "bool_lit_view");
    assert!(bool_lit.ok(), "{:?}", bool_lit.issues);

    let ne = compile("@if (label != 'x') { <span>y</span> }", "ne_view");
    assert!(ne.ok(), "{:?}", ne.issues);

    let add = compile("<span>{{label + n}}</span>", "add_view");
    assert!(add.ok(), "{:?}", add.issues);

    let tokens = compile_tokens("<p>{{label}}</p>", "tok_view");
    assert!(tokens.ok(), "{:?}", tokens.issues);
    let src = tokens_to_rust_source(&tokens.tokens);
    assert_ne!(src, "");
}

struct FailHost;

impl Host for FailHost {
    fn get(&self, _: &str) -> Option<Value> {
        None
    }
    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        Err(HostError::UnknownMember(name.to_owned()))
    }
}

#[test]
fn host_cell_event_item_none_call_err_and_num_lit() {
    use rangular_aot::compile;

    let cell = HostCell::new(MapHost::new());
    assert_eq!(cell.eval_value(&Expr::Ident("$event".into())), Value::Unit);

    let scope = LoopScope {
        item_name: Some("item"),
        item_val: None,
        index: Some(0),
        count: Some(1),
    };
    assert_eq!(cell.eval_scoped(&expr("item"), scope), Value::Unit);

    let fail = HostCell::new(FailHost);
    assert_eq!(
        fail.eval_scoped(&expr("onTap()"), LoopScope::NONE),
        Value::Unit
    );
    let nested = Expr::Call {
        callee: Box::new(Expr::Call {
            callee: Box::new(Expr::Ident("foo".into())),
            args: vec![],
        }),
        args: vec![],
    };
    assert_eq!(fail.eval_scoped(&nested, LoopScope::NONE), Value::Unit);

    let num = compile("@if (1) { <span>y</span> }", "num_lit_view");
    assert!(num.ok(), "{:?}", num.issues);
}
