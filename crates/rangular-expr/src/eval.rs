use rangular_host::{Host, HostError, Value};

use crate::ast::{BinOp, Expr, Literal, UnOp};
use crate::pipe::PipeRegistry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalError {
    UnknownIdent(String),
    UnknownPipe(String),
    BadCallee,
    TypeMismatch(&'static str),
    Host(HostError),
}

/// Evaluate `expr` against `host` using builtin pipes.
///
/// # Errors
///
/// When an identifier is missing, types do not match, a pipe is unknown, or the
/// host rejects a call.
pub fn eval(expr: &Expr, host: &mut impl Host) -> Result<Value, EvalError> {
    eval_with_pipes(expr, host, PipeRegistry::builtins())
}

/// Evaluate `expr` with an explicit [`PipeRegistry`] (builtins and/or custom).
///
/// # Errors
///
/// When an identifier is missing, types do not match, a pipe is unknown, or the
/// host rejects a call.
pub fn eval_with_pipes(
    expr: &Expr,
    host: &mut impl Host,
    pipes: &PipeRegistry,
) -> Result<Value, EvalError> {
    match expr {
        Expr::Lit(lit) => Ok(lit_to_value(lit)),
        Expr::Ident(name) => host
            .get(name)
            .ok_or_else(|| EvalError::UnknownIdent(name.clone())),
        Expr::Unary {
            op: UnOp::Not,
            expr,
        } => Ok(Value::Bool(
            !eval_with_pipes(expr, host, pipes)?.is_truthy(),
        )),
        Expr::Binary { op, left, right } => eval_binary(*op, left, right, host, pipes),
        Expr::Call { callee, args } => {
            let name = call_name(callee)?;
            let values = args
                .iter()
                .map(|arg| eval_with_pipes(arg, host, pipes))
                .collect::<Result<Vec<_>, _>>()?;
            host.call(&name, &values).map_err(EvalError::Host)
        }
        Expr::Pipe { expr, name, args } => {
            let left = eval_with_pipes(expr, host, pipes)?;
            let arg_vals = args
                .iter()
                .map(|arg| eval_with_pipes(arg, host, pipes))
                .collect::<Result<Vec<_>, _>>()?;
            pipes.apply(name, &left, &arg_vals)
        }
        Expr::Ternary {
            cond,
            then_branch,
            else_branch,
        } => {
            if eval_with_pipes(cond, host, pipes)?.is_truthy() {
                eval_with_pipes(then_branch, host, pipes)
            } else {
                eval_with_pipes(else_branch, host, pipes)
            }
        }
    }
}

fn eval_binary(
    op: BinOp,
    left: &Expr,
    right: &Expr,
    host: &mut impl Host,
    pipes: &PipeRegistry,
) -> Result<Value, EvalError> {
    match op {
        BinOp::Or => {
            let l = eval_with_pipes(left, host, pipes)?;
            if l.is_truthy() {
                return Ok(l);
            }
            eval_with_pipes(right, host, pipes)
        }
        BinOp::And => {
            let l = eval_with_pipes(left, host, pipes)?;
            if !l.is_truthy() {
                return Ok(l);
            }
            eval_with_pipes(right, host, pipes)
        }
        BinOp::Eq => Ok(Value::Bool(values_equal(
            &eval_with_pipes(left, host, pipes)?,
            &eval_with_pipes(right, host, pipes)?,
        ))),
        BinOp::Ne => Ok(Value::Bool(!values_equal(
            &eval_with_pipes(left, host, pipes)?,
            &eval_with_pipes(right, host, pipes)?,
        ))),
        BinOp::Add => add_values(
            &eval_with_pipes(left, host, pipes)?,
            &eval_with_pipes(right, host, pipes)?,
        ),
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
