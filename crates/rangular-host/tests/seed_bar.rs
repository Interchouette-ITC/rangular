use rangular_expr::{eval, parse, Expr, Host, Value};
use rangular_host::HostError;

struct SeedBarHost {
    seed: String,
    worker_ready: bool,
    busy: bool,
    generated_for: String,
    event: Value,
    last_call: Option<String>,
}

impl SeedBarHost {
    fn generate_disabled(&self) -> bool {
        !self.worker_ready
            || self.busy
            || (!self.generated_for.is_empty() && self.generated_for == self.seed)
    }

    const fn random_disabled(&self) -> bool {
        !self.worker_ready || self.busy
    }
}

impl Host for SeedBarHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "seed" => Some(Value::Str(self.seed.clone())),
            "generateDisabled" => Some(Value::Bool(self.generate_disabled())),
            "randomDisabled" => Some(Value::Bool(self.random_disabled())),
            "$event" => Some(self.event.clone()),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, HostError> {
        self.last_call = Some(format!("{name}({args:?})"));
        Ok(Value::Unit)
    }
}

struct IdHost;

impl Host for IdHost {
    fn get(&self, name: &str) -> Option<Value> {
        (name == "inputId").then(|| Value::Str("id1".into()))
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

fn must_parse(input: &str) -> Expr {
    parse(input).expr.expect("expr")
}

#[test]
fn seed_bar_generate_disabled() {
    let mut host = SeedBarHost {
        seed: "abc".into(),
        worker_ready: true,
        busy: false,
        generated_for: "abc".into(),
        event: Value::Str("evt".into()),
        last_call: None,
    };
    let expr = must_parse("generateDisabled");
    assert_eq!(eval(&expr, &mut host).unwrap(), Value::Bool(true));

    host.generated_for.clear();
    assert_eq!(eval(&expr, &mut host).unwrap(), Value::Bool(false));
}

#[test]
fn seed_bar_handlers() {
    let mut host = SeedBarHost {
        seed: "abc".into(),
        worker_ready: true,
        busy: false,
        generated_for: String::new(),
        event: Value::Str("input".into()),
        last_call: None,
    };

    eval(&must_parse("onGenerate()"), &mut host).unwrap();
    assert_eq!(host.last_call.as_deref(), Some("onGenerate([])"));

    eval(&must_parse("seedChange($event)"), &mut host).unwrap();
    assert!(host
        .last_call
        .as_deref()
        .is_some_and(|c| c.starts_with("seedChange(")));
}

#[test]
fn seed_bar_string_concat() {
    let mut host = IdHost;
    let expr = must_parse("inputId + '-hex'");
    assert_eq!(
        eval(&expr, &mut host).unwrap(),
        Value::Str("id1-hex".into())
    );
}

#[test]
fn seed_bar_random_disabled_when_busy() {
    let mut host = SeedBarHost {
        seed: String::new(),
        worker_ready: true,
        busy: true,
        generated_for: String::new(),
        event: Value::Unit,
        last_call: None,
    };
    assert_eq!(
        eval(&must_parse("randomDisabled"), &mut host).unwrap(),
        Value::Bool(true)
    );
}
