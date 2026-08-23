use rangular_expr::{eval, parse, Expr, Host, Value};
use rangular_host::HostError;

struct Counter;

impl Host for Counter {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "n" => Some(Value::Num(2.0)),
            "flag" => Some(Value::Bool(false)),
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
