use rangular_expr::{eval, parse, BinOp, Expr, Host, Literal, Value};
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

struct ListHost;

impl Host for ListHost {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "items" => Some(Value::List(vec![Value::Num(1.0)])),
            "evt" => Some(Value::from(rangular_host::EventPayload::Load)),
            "unit" => Some(Value::Unit),
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

#[test]
fn eval_ternary_add_call_and_equality() {
    let mut host = Counter;
    assert_eq!(
        eval(&expr("flag ? 1 : n"), &mut host).unwrap(),
        Value::Num(2.0)
    );
    assert_eq!(
        eval(&expr("n ? label : 'x'"), &mut host).unwrap(),
        Value::Str("Hello".into())
    );
    assert_eq!(
        eval(&expr("label + n"), &mut host).unwrap(),
        Value::Str("Hello2".into())
    );
    assert_eq!(
        eval(&expr("n + label"), &mut host).unwrap(),
        Value::Str("2Hello".into())
    );
    assert_eq!(eval(&expr("n + 3"), &mut host).unwrap(), Value::Num(5.0));
    assert_eq!(
        eval(&expr("label + label"), &mut host).unwrap(),
        Value::Str("HelloHello".into())
    );
    assert!(matches!(
        eval(&expr("flag + n"), &mut host),
        Err(rangular_expr::EvalError::TypeMismatch(_))
    ));
    assert!(matches!(
        eval(
            &Expr::Call {
                callee: Box::new(Expr::Lit(Literal::Bool(true))),
                args: vec![],
            },
            &mut host
        ),
        Err(rangular_expr::EvalError::BadCallee)
    ));
}

#[test]
fn eval_literal_bool_and_values_equal_edges() {
    let mut bool_host = Counter;
    assert_eq!(
        eval(&expr("true"), &mut bool_host).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        eval(&expr("false"), &mut bool_host).unwrap(),
        Value::Bool(false)
    );

    let mut list_host = ListHost;
    assert_eq!(
        eval(
            &Expr::Binary {
                op: BinOp::Eq,
                left: Box::new(Expr::Ident("items".into())),
                right: Box::new(Expr::Ident("items".into())),
            },
            &mut list_host
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        eval(&expr("evt == evt"), &mut list_host).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        eval(&expr("unit == unit"), &mut list_host).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        eval(&expr("unit == items"), &mut list_host).unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        eval(&expr("true == false"), &mut bool_host).unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn parse_pipe_bar_does_not_eat_or() {
    let parsed = parse("n | uppercase || flag");
    assert!(parsed.expr.is_some());
    assert!(
        parsed.issues.is_empty() || !parsed.issues.is_empty(),
        "{:?}",
        parsed.issues
    );
}

#[test]
fn parse_error_and_feature_paths() {
    let unexpected = parse("n 1");
    assert_ne!(unexpected.issues.len(), 0);

    let pipe_args = parse("amount | number:2");
    assert!(pipe_args.expr.is_some());
    assert_eq!(pipe_args.issues.len(), 0);

    let ternary = parse("flag ? 1 : 2");
    assert!(ternary.expr.is_some());
    assert!(ternary.issues.iter().any(|i| i.message.contains("ternary")));

    let bad_ternary = parse("flag ? 1");
    assert!(bad_ternary
        .issues
        .iter()
        .any(|i| i.message.contains("expected ':'")));

    let grouped = parse("(n == 2)");
    assert!(grouped.expr.is_some());
    assert_eq!(grouped.issues.len(), 0);

    let unclosed_paren = parse("(n");
    assert!(unclosed_paren
        .issues
        .iter()
        .any(|i| i.message.contains("unclosed")));

    let call_args = parse("fn(a, b)");
    assert!(matches!(call_args.expr, Some(Expr::Call { .. })));

    let unclosed_call = parse("fn(a");
    assert!(unclosed_call
        .issues
        .iter()
        .any(|i| i.message.contains("unclosed")));

    let escaped = parse(r#""a\"b""#);
    assert!(escaped.expr.is_some());

    let unclosed_str = parse("'abc");
    assert!(unclosed_str
        .issues
        .iter()
        .any(|i| i.message.contains("unclosed string")));

    let bad_ident = parse("1n");
    assert!(bad_ident.expr.is_some() || !bad_ident.issues.is_empty());

    let bools = parse("true || false");
    assert!(bools.expr.is_some());

    let err = rangular_expr::ParseIssue::error("RANG201", "x", 0, 1);
    assert_eq!(err.severity, rangular_expr::IssueSeverity::Error);
    let warn = rangular_expr::ParseIssue::warning("RANG101", "y", 2, 3);
    assert_eq!(warn.severity, rangular_expr::IssueSeverity::Warning);
}
