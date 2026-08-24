use rangular_expr::{eval, parse};
use rangular_host::{EventPayload, Host, HostError, Value};

struct CaptureHost {
    event: Value,
    last: Option<(String, Vec<Value>)>,
}

impl Host for CaptureHost {
    fn get(&self, name: &str) -> Option<Value> {
        (name == "$event").then(|| self.event.clone())
    }

    fn set(&mut self, name: &str, value: Value) -> Result<(), HostError> {
        if name == "$event" {
            self.event = value;
        }
        Ok(())
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        self.last = Some((name.to_owned(), args.to_vec()));
        Ok(Value::Unit)
    }
}

#[test]
fn from_dom_click_and_input() {
    assert_eq!(
        EventPayload::from_dom("click", String::new()),
        EventPayload::Click
    );
    assert_eq!(
        EventPayload::from_dom("input", "abc".into()),
        EventPayload::Input {
            value: "abc".into()
        }
    );
    assert_eq!(
        EventPayload::from_dom("error", String::new()),
        EventPayload::Error
    );
}

#[test]
fn event_payload_as_str_for_input() {
    let v = Value::from(EventPayload::Input {
        value: "hello".into(),
    });
    assert_eq!(v.as_str(), Some("hello"));
    assert!(v.is_truthy());
}

#[test]
fn seed_change_reads_typed_event() {
    let mut host = CaptureHost {
        event: Value::from(EventPayload::Input {
            value: "seed-1".into(),
        }),
        last: None,
    };
    eval(&parse("seedChange($event)").expr.expect("expr"), &mut host).unwrap();
    let (name, args) = host.last.expect("call");
    assert_eq!(name, "seedChange");
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].as_str(), Some("seed-1"));
    assert!(matches!(
        args[0].as_event(),
        Some(EventPayload::Input { value }) if value == "seed-1"
    ));
}
