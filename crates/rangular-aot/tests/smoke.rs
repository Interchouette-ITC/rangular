use rangular_aot::compile;
use syn::parse_file;

fn assert_emits(html: &str, fn_name: &str, label: &str) {
    let out = compile(html, fn_name);
    assert!(out.ok(), "{label}: {:?}", out.issues);
    assert_ne!(out.code, "", "{label}: expected emitted Rust");
    parse_file(&out.code).unwrap_or_else(|err| panic!("{label}: invalid Rust: {err}"));
}

#[test]
fn seed_bar_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/seed-bar/seed-bar.html");
    assert_emits(html, "seed_bar_view", "seed-bar");
}

#[test]
fn asset_icon_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/asset-icon/asset-icon.html");
    assert_emits(html, "asset_icon_view", "asset-icon");
}

#[test]
fn color_field_parse_has_nodes() {
    let html = include_str!("../../../tests/fixtures/components/color-field/color-field.html");
    assert!(html.contains("color-field"));
    let parsed = rangular_parser::parse(html, "color-field.html");
    assert!(parsed.ok(), "{:?}", parsed.diagnostics);
    assert!(!parsed.template.nodes.is_empty(), "expected root nodes");
}

#[test]
fn chrome_header_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/chrome-header/chrome-header.html");
    assert_emits(html, "chrome_header_view", "chrome-header");
}

#[test]
fn color_field_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/color-field/color-field.html");
    assert_emits(html, "color_field_view", "color-field");
}

#[test]
fn layout_shell_emits_children_slot() {
    let html = include_str!("../../../tests/fixtures/components/layout-shell/layout-shell.html");
    let out = compile(html, "layout_shell_view");
    assert!(out.ok(), "{:?}", out.issues);
    assert!(
        out.code.contains("children:Children") || out.code.contains("children: Children"),
        "expected Children param:\n{}",
        out.code
    );
    assert!(
        out.code.contains("children()") || out.code.contains("children ()"),
        "expected children() projection:\n{}",
        out.code
    );
    parse_file(&out.code).unwrap_or_else(|err| panic!("invalid Rust: {err}"));
}

#[test]
fn two_way_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/two-way/two-way.html");
    assert_emits(html, "two_way_view", "two-way");
    let out = compile(html, "two_way_view");
    assert!(
        out.code.contains("input") && out.code.contains("$bananaSet"),
        "expected banana desugar in emit:\n{}",
        out.code
    );
}

#[test]
fn named_slots_emits_slot_params() {
    let html = include_str!("../../../tests/fixtures/components/named-slots/named-slots.html");
    assert_emits(html, "named_slots_view", "named-slots");
    let out = compile(html, "named_slots_view");
    assert!(
        out.code.contains("slot_header: Children") || out.code.contains("slot_header:Children"),
        "expected slot_header param:\n{}",
        out.code
    );
}

#[test]
fn template_outlet_emits_stamped_body() {
    let html =
        include_str!("../../../tests/fixtures/components/template-outlet/template-outlet.html");
    assert_emits(html, "template_outlet_view", "template-outlet");
    let out = compile(html, "template_outlet_view");
    assert!(
        out.code.contains("card") || out.code.contains("label"),
        "expected stamped template body:\n{}",
        out.code
    );
}

#[test]
fn banana_hostcell_sets_via_dom_event() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rangular_aot::HostCell;
    use rangular_expr::{Expr, Host, Value};
    use rangular_host::HostError;
    use rangular_parser::banana_write_expr;

    struct SeedHost {
        seed: Rc<RefCell<String>>,
    }

    impl Host for SeedHost {
        fn get(&self, name: &str) -> Option<Value> {
            (name == "seed").then(|| Value::Str(self.seed.borrow().clone()))
        }

        fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
            if name == "seed" {
                if let Some(s) = value.as_str() {
                    *self.seed.borrow_mut() = s.to_owned();
                }
            }
            Ok(())
        }

        fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
            Ok(Value::Unit)
        }
    }

    let seed = Rc::new(RefCell::new("abc".into()));
    let cell = HostCell::new(SeedHost {
        seed: Rc::clone(&seed),
    });
    let write = banana_write_expr(&Expr::Ident("seed".into()));
    cell.emit_dom_event_call("$bananaSet", &write, "input", "xyz".into());
    assert_eq!(*seed.borrow(), "xyz");
}

#[test]
fn hostcell_dom_event_passes_dollar_event_to_handler() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rangular_aot::HostCell;
    use rangular_expr::{parse, Host, Value};
    use rangular_host::{EventPayload, HostError};

    struct CaptureHost {
        args: Rc<RefCell<Vec<Value>>>,
    }

    impl Host for CaptureHost {
        fn get(&self, _: &str) -> Option<Value> {
            None
        }

        fn call(&mut self, _: &str, args: &[Value]) -> Result<Value, HostError> {
            *self.args.borrow_mut() = args.to_vec();
            Ok(Value::Unit)
        }
    }

    let args = Rc::new(RefCell::new(Vec::new()));
    let cell = HostCell::new(CaptureHost {
        args: Rc::clone(&args),
    });
    let handler = parse("onInput($event)").expr.expect("expr");
    cell.emit_dom_event_call("onInput", &handler, "input", "typed".into());
    let got = args.borrow().clone();
    assert_eq!(got.len(), 1);
    assert!(matches!(
        got[0].as_event(),
        Some(EventPayload::Input { value }) if value == "typed"
    ));
}

#[test]
fn garbage_input_returns_issues_not_empty_code() {
    let out = compile("<@broken", "broken_view");
    assert!(!out.ok());
    assert_eq!(out.code, "");
}
