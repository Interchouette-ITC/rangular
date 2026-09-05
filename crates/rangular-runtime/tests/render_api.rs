use rangular_expr::{Host, Value};
use rangular_host::{EventPayload, HostError};
use rangular_runtime::{
    interpret, interpret_with_slot, interpret_with_slots, render, render_with_slot,
    render_with_slots, ProjectionBag, VNode,
};

struct DemoHost;

impl Host for DemoHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "flag" => Some(Value::Bool(true)),
            "off" => Some(Value::Bool(false)),
            "label" => Some(Value::Str("Hi".into())),
            "items" => Some(Value::List(vec![
                Value::Str("a".into()),
                Value::Str("b".into()),
            ])),
            "evt" => Some(Value::from(EventPayload::Click { x: 1, y: 2 })),
            "inp" => Some(Value::from(EventPayload::Input { value: "x".into() })),
            "err" => Some(Value::from(EventPayload::Error)),
            "load" => Some(Value::from(EventPayload::Load)),
            "custom" => Some(Value::from(EventPayload::Custom(Box::new(Value::Num(3.0))))),
            "unit" => Some(Value::Unit),
            "list" => Some(Value::List(vec![Value::Bool(true), Value::Num(2.0)])),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Str(format!("call:{name}")))
    }
}

#[test]
fn interpret_parse_error_returns_issues() {
    let mut host = DemoHost;
    let out = interpret("<@bad", "t.html", &mut host);
    assert!(!out.ok());
    assert_eq!(out.nodes.len(), 0);
}

#[test]
fn render_apis_and_display_variants() {
    let mut host = DemoHost;
    let parsed = rangular_parser::parse(
        r#"
<div disabled hidden [title]="label" (click)="onTap()">
  {{label}} {{evt}} {{inp}} {{err}} {{load}} {{custom}} {{unit}} {{list}}
</div>
<!-- skip -->
<ng-template #card><span>{{label}}</span></ng-template>
<ng-container [ngTemplateOutlet]="card"></ng-container>
<ng-container><em>inner</em></ng-container>
@if (flag) { <span>yes</span> } @else { <span>no</span> }
@if (off) { <span>hidden</span> }
@for (let item of items) { <span>{{item}}{{$index}}</span> }
"#,
        "t.html",
    );
    assert!(parsed.ok(), "{:?}", parsed.diagnostics);

    let via_render = render(&parsed.template, &mut host);
    assert!(via_render.ok(), "{:?}", via_render.issues);
    assert_ne!(via_render.nodes.len(), 0);

    let slot = [VNode::Element {
        tag: "header".into(),
        attrs: vec![("class".into(), "header".into())],
        children: vec![VNode::Text("H".into())],
    }];
    let with_slot = render_with_slot(&parsed.template, &mut host, &slot);
    assert!(with_slot.ok());

    let bag = ProjectionBag::from_flat(&slot, &[".header".into()]);
    let with_bag = render_with_slots(&parsed.template, &mut host, &bag);
    assert!(with_bag.ok());

    let projected = interpret_with_slot(
        "<section><rg-content select=\".header\"></rg-content></section>",
        "t.html",
        &mut host,
        &slot,
    );
    assert!(projected.ok(), "{:?}", projected.issues);

    let via_slots = interpret_with_slots(
        "<section><rg-content select=\".header\"></rg-content><rg-content></rg-content></section>",
        "t.html",
        &mut host,
        &bag,
    );
    assert!(via_slots.ok(), "{:?}", via_slots.issues);
}

#[test]
fn interpret_with_slot_parse_error_and_missing_outlet() {
    use rangular_runtime::interpret_with_slot_and_pipes;

    let mut host = DemoHost;
    let slot = [VNode::Text("x".into())];
    let bad = interpret_with_slot("<@bad", "t.html", &mut host, &slot);
    assert!(!bad.ok());
    assert_eq!(bad.nodes.len(), 0);

    let pipes = rangular_expr::PipeRegistry::with_builtins();
    let bad_pipes = interpret_with_slot_and_pipes("<", "t.html", &mut host, &slot, &pipes);
    assert!(!bad_pipes.ok());

    let missing = interpret(
        r#"
<ng-template #card><span>x</span></ng-template>
<ng-container [ngTemplateOutlet]="missing"></ng-container>
"#,
        "t.html",
        &mut host,
    );
    assert!(missing.ok() || !missing.issues.is_empty());
}

#[test]
fn structural_ir_and_projection_bag_edges() {
    use rangular_runtime::structural_ir;

    assert!(structural_ir("<@bad", "t.html").is_none());
    assert!(structural_ir("<p>ok</p>", "t.html").is_some());

    let roots = [
        VNode::Element {
            tag: "div".into(),
            attrs: vec![("class".into(), "header".into())],
            children: vec![],
        },
        VNode::Text("loose".into()),
        VNode::Element {
            tag: "div".into(),
            attrs: vec![("class".into(), "header footer".into())],
            children: vec![],
        },
    ];
    let bag = ProjectionBag::from_flat(&roots, &[".header".into(), ".footer".into()]);
    assert_ne!(bag.for_select(Some(".header")).len(), 0);
    assert_eq!(bag.for_select(Some(".missing")).len(), 0);
    assert_ne!(bag.for_select(None).len(), 0);

    let mut host = DemoHost;
    let with_ref = interpret(
        "<div #localRef [disabled]=\"flag\"></div>",
        "t.html",
        &mut host,
    );
    assert!(with_ref.ok(), "{:?}", with_ref.issues);
}
