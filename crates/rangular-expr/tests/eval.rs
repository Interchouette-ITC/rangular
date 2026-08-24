use rangular_expr::{eval, parse, Expr, Host, Value};
use rangular_host::HostError;

struct Counter;

impl Host for Counter {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "n" => Some(Value::Num(2.0)),
            "flag" => Some(Value::Bool(false)),
            "label" => Some(Value::Str("Hello".into())),
            "amount" => Some(Value::Num(42.5)),
            _ => None,
        }
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}

fn expr(input: &str) -> Expr {
    parse(input).expr.unwrap()
}

#[test]
fn eval_logic_and_compare() {
    let mut host = Counter;
    assert_eq!(
        eval(&expr("n == 2 && !flag"), &mut host).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        eval(&expr("n != 3 || flag"), &mut host).unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn eval_unknown_ident() {
    let mut host = Counter;
    assert!(eval(&expr("missing"), &mut host).is_err());
}

#[test]
fn eval_builtin_pipes() {
    let mut host = Counter;
    assert_eq!(
        eval(&expr("label | uppercase"), &mut host).unwrap(),
        Value::Str("HELLO".into())
    );
    assert_eq!(
        eval(&expr("label | lowercase"), &mut host).unwrap(),
        Value::Str("hello".into())
    );
    assert_eq!(
        eval(&expr("amount | number"), &mut host).unwrap(),
        Value::Str("42.5".into())
    );
    assert_eq!(
        eval(&expr("label | json"), &mut host).unwrap(),
        Value::Str("\"Hello\"".into())
    );
    assert_eq!(
        eval(&expr("label | lowercase | uppercase"), &mut host).unwrap(),
        Value::Str("HELLO".into())
    );
}

#[test]
fn parse_pipe_not_or() {
    let parsed = parse("n == 2 || flag");
    assert_eq!(parsed.issues.len(), 0);
    assert!(matches!(parsed.expr, Some(Expr::Binary { .. })));
}
