use std::sync::{Arc, OnceLock};

use rangular_expr::{EvalError, PipeRegistry};
use rangular_host::Value;

pub fn demo_pipes() -> Arc<PipeRegistry> {
    static PIPES: OnceLock<Arc<PipeRegistry>> = OnceLock::new();
    Arc::clone(PIPES.get_or_init(|| {
        let mut reg = PipeRegistry::with_builtins();
        reg.register("crab", pipe_crab);
        Arc::new(reg)
    }))
}

#[allow(clippy::unnecessary_wraps)]
fn pipe_crab(value: &Value, _: &[Value]) -> Result<Value, EvalError> {
    let text = match value {
        Value::Str(s) => s.clone(),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::Str(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(", "),
        Value::Event(payload) => format!("{payload:?}"),
        Value::Unit => String::new(),
    };
    Ok(Value::Str(format!("{text} 🦀")))
}

#[cfg(test)]
mod tests {
    use super::demo_pipes;
    use rangular_expr::{eval_with_pipes, parse};
    use rangular_host::{Host, HostError, Value};

    struct LabelHost;

    impl Host for LabelHost {
        fn get(&self, name: &str) -> Option<Value> {
            (name == "label").then(|| Value::Str("Hello".into()))
        }

        fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
            Ok(Value::Unit)
        }
    }

    #[test]
    fn crab_pipe_appends_emoji() {
        let expr = parse("label | crab").expr.expect("pipe expr");
        let mut host = LabelHost;
        let out = eval_with_pipes(&expr, &mut host, &demo_pipes()).unwrap();
        assert_eq!(out, Value::Str("Hello 🦀".into()));
    }
}
