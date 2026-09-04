use rangular_host::{EventPayload, Value};

#[test]
fn is_truthy_covers_variants() {
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(Value::Str("x".into()).is_truthy());
    assert!(!Value::Str(String::new()).is_truthy());
    assert!(Value::Num(1.0).is_truthy());
    assert!(!Value::Num(0.0).is_truthy());
    assert!(Value::List(vec![Value::Unit]).is_truthy());
    assert!(!Value::List(vec![]).is_truthy());
    assert!(!Value::Unit.is_truthy());
    assert!(Value::from(EventPayload::Click { x: 1, y: 2 }).is_truthy());
    assert!(Value::from(EventPayload::Error).is_truthy());
    assert!(Value::from(EventPayload::Load).is_truthy());
    assert!(Value::from(EventPayload::Input { value: "a".into() }).is_truthy());
    assert!(!Value::from(EventPayload::Input {
        value: String::new()
    })
    .is_truthy());
    assert!(Value::from(EventPayload::Custom(Box::new(Value::Bool(true)))).is_truthy());
    assert!(!Value::from(EventPayload::Custom(Box::new(Value::Unit))).is_truthy());
}

#[test]
fn as_str_as_bool_as_event() {
    assert_eq!(Value::Str("hi".into()).as_str(), Some("hi"));
    assert_eq!(
        Value::from(EventPayload::Input {
            value: "typed".into()
        })
        .as_str(),
        Some("typed")
    );
    assert_eq!(
        Value::from(EventPayload::Custom(Box::new(Value::Str("c".into())))).as_str(),
        Some("c")
    );
    assert_eq!(Value::Num(1.0).as_str(), None);
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Str("x".into()).as_bool(), None);
    assert!(Value::from(EventPayload::Load).as_event().is_some());
    assert!(Value::Unit.as_event().is_none());
}

#[test]
fn from_impls() {
    assert_eq!(Value::from(true), Value::Bool(true));
    assert_eq!(Value::from("ab"), Value::Str("ab".into()));
    assert_eq!(Value::from(String::from("cd")), Value::Str("cd".into()));
}

#[test]
fn event_payload_labels_and_from_dom() {
    assert_eq!(
        EventPayload::from_dom("click", "3,4".into()),
        EventPayload::Click { x: 3, y: 4 }
    );
    assert_eq!(
        EventPayload::from_dom("dblclick", "bad".into()),
        EventPayload::Click { x: 0, y: 0 }
    );
    assert_eq!(
        EventPayload::from_dom("change", "v".into()),
        EventPayload::Input { value: "v".into() }
    );
    assert_eq!(
        EventPayload::from_dom("error", String::new()),
        EventPayload::Error
    );
    assert_eq!(
        EventPayload::from_dom("load", String::new()),
        EventPayload::Load
    );
    assert_eq!(
        EventPayload::from_dom("focus", "x".into()),
        EventPayload::Custom(Box::new(Value::Str("x".into())))
    );
    assert_eq!(
        EventPayload::Click { x: 1, y: 2 }.demo_label(),
        "Click { x: 1, y: 2 }"
    );
    assert_eq!(
        EventPayload::Input { value: "a".into() }.demo_label(),
        "Input { value: \"a\" }"
    );
    assert_eq!(EventPayload::Error.demo_label(), "Error");
    assert_eq!(EventPayload::Load.demo_label(), "Load");
    assert!(EventPayload::Custom(Box::new(Value::Unit))
        .demo_label()
        .starts_with("Custom("));
    assert_eq!(EventPayload::Click { x: 0, y: 0 }.kind_label(), "click");
    assert_eq!(
        EventPayload::Input {
            value: String::new()
        }
        .kind_label(),
        "input"
    );
    assert_eq!(EventPayload::Error.kind_label(), "error");
    assert_eq!(EventPayload::Load.kind_label(), "load");
    assert_eq!(
        EventPayload::Custom(Box::new(Value::Unit)).kind_label(),
        "custom"
    );
}
