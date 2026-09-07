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

fn pipe_crab(value: &Value, args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::TypeMismatch("crab pipe args"));
    }
    let text = match value {
        Value::Str(s) => s.clone(),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => {
            let mut parts = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::Str(s) => parts.push(s.as_str()),
                    _ => return Err(EvalError::TypeMismatch("crab pipe list")),
                }
            }
            parts.join(", ")
        }
        Value::Event(_) | Value::Unit => {
            return Err(EvalError::TypeMismatch("crab pipe"));
        }
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

    #[test]
    fn crab_pipe_rejects_unit() {
        let pipes = demo_pipes();
        let err = pipes
            .apply("crab", &Value::Unit, &[])
            .expect_err("unit");
        assert!(matches!(err, rangular_expr::EvalError::TypeMismatch(_)));
    }
}
