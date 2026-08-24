use rangular_host::{Host, HostError, Value};

use crate::ast::{BinOp, Expr, Literal, UnOp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalError {
    UnknownIdent(String),
    BadCallee,
    TypeMismatch(&'static str),
    Host(HostError),
}

/// # Errors
///
/// When an identifier is missing, types do not match, or the host rejects a call.
pub fn eval(expr: &Expr, host: &mut impl Host) -> Result<Value, EvalError> {
    match expr {
        Expr::Lit(lit) => Ok(lit_to_value(lit)),
        Expr::Ident(name) => host
            .get(name)
            .ok_or_else(|| EvalError::UnknownIdent(name.clone())),
        Expr::Unary {
            op: UnOp::Not,
            expr,
        } => Ok(Value::Bool(!eval(expr, host)?.is_truthy())),
        Expr::Binary { op, left, right } => eval_binary(*op, left, right, host),
        Expr::Call { callee, args } => {
            let name = call_name(callee)?;
            let values = args
                .iter()
                .map(|arg| eval(arg, host))
                .collect::<Result<Vec<_>, _>>()?;
            host.call(&name, &values).map_err(EvalError::Host)
        }
        Expr::Ternary {
            cond,
            then_branch,
            else_branch,
        } => {
            if eval(cond, host)?.is_truthy() {
                eval(then_branch, host)
            } else {
                eval(else_branch, host)
            }
        }
    }
}

fn eval_binary(
    op: BinOp,
    left: &Expr,
    right: &Expr,
    host: &mut impl Host,
) -> Result<Value, EvalError> {
    match op {
        BinOp::Or => {
            let l = eval(left, host)?;
            if l.is_truthy() {
                return Ok(l);
            }
            eval(right, host)
        }
        BinOp::And => {
            let l = eval(left, host)?;
            if !l.is_truthy() {
                return Ok(l);
            }
            eval(right, host)
        }
        BinOp::Eq => Ok(Value::Bool(values_equal(
            &eval(left, host)?,
            &eval(right, host)?,
        ))),
        BinOp::Ne => Ok(Value::Bool(!values_equal(
            &eval(left, host)?,
            &eval(right, host)?,
        ))),
        BinOp::Add => add_values(&eval(left, host)?, &eval(right, host)?),
    }
}

fn call_name(callee: &Expr) -> Result<String, EvalError> {
    match callee {
        Expr::Ident(name) => Ok(name.clone()),
        _ => Err(EvalError::BadCallee),
    }
}

fn lit_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Str(s) => Value::Str(s.clone()),
        Literal::Num(n) => Value::Num(*n),
        Literal::Bool(b) => Value::Bool(*b),
    }
}

fn add_values(left: &Value, right: &Value) -> Result<Value, EvalError> {
    match (left, right) {
        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
        (Value::Str(a), Value::Num(b)) => Ok(Value::Str(format!("{a}{b}"))),
        (Value::Num(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
        (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
        _ => Err(EvalError::TypeMismatch("addition")),
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Num(a), Value::Num(b)) => (a - b).abs() < f64::EPSILON,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::List(a), Value::List(b)) => a == b,
        (Value::Event(a), Value::Event(b)) => a == b,
        (Value::Unit, Value::Unit) => true,
        _ => false,
    }
}
