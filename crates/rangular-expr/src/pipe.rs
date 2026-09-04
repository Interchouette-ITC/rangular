use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use rangular_host::Value;

use crate::eval::EvalError;

/// Pure pipe transform: input value plus optional colon args.
pub type PipeFn = fn(&Value, &[Value]) -> Result<Value, EvalError>;

/// Stringly name → pipe function map (builtins + app custom).
#[derive(Clone, Default)]
pub struct PipeRegistry {
    pipes: HashMap<String, PipeFn>,
}

impl PipeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builtins: `uppercase`, `lowercase`, `number`, `json`.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        reg.register("uppercase", pipe_uppercase);
        reg.register("lowercase", pipe_lowercase);
        reg.register("number", pipe_number);
        reg.register("json", pipe_json);
        reg
    }

    /// Process-wide builtins registry.
    #[must_use]
    pub fn builtins() -> &'static Self {
        static BUILTINS: OnceLock<PipeRegistry> = OnceLock::new();
        BUILTINS.get_or_init(Self::with_builtins)
    }

    /// Shared builtins handle for `HostCell` / runtime.
    #[must_use]
    pub fn builtins_arc() -> Arc<Self> {
        static BUILTINS_ARC: OnceLock<Arc<PipeRegistry>> = OnceLock::new();
        BUILTINS_ARC
            .get_or_init(|| Arc::new(Self::with_builtins()))
            .clone()
    }

    pub fn register(&mut self, name: impl Into<String>, pipe: PipeFn) {
        self.pipes.insert(name.into(), pipe);
    }

    /// # Errors
    ///
    /// When the pipe name is unknown or the transform rejects the value/args.
    pub fn apply(&self, name: &str, value: &Value, args: &[Value]) -> Result<Value, EvalError> {
        let pipe = self
            .pipes
            .get(name)
            .ok_or_else(|| EvalError::UnknownPipe(name.to_owned()))?;
        pipe(value, args)
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.pipes.contains_key(name)
    }
}

fn display_plain(value: &Value) -> String {
    match value {
        Value::Str(s) => s.clone(),
        Value::Num(n) => format_num(*n),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => items
            .iter()
            .map(display_plain)
            .collect::<Vec<_>>()
            .join(","),
        Value::Event(payload) => format!("{payload:?}"),
        Value::Unit => String::new(),
    }
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        n.to_string()
    }
}

fn as_number(value: &Value) -> Result<f64, EvalError> {
    match value {
        Value::Num(n) => Ok(*n),
        Value::Str(s) => s
            .parse::<f64>()
            .map_err(|_| EvalError::TypeMismatch("number pipe")),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err(EvalError::TypeMismatch("number pipe")),
    }
}

#[allow(clippy::unnecessary_wraps)] // PipeFn signature is always Result
fn pipe_uppercase(value: &Value, _: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Str(display_plain(value).to_uppercase()))
}

#[allow(clippy::unnecessary_wraps)] // PipeFn signature is always Result
fn pipe_lowercase(value: &Value, _: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Str(display_plain(value).to_lowercase()))
}

fn pipe_number(value: &Value, args: &[Value]) -> Result<Value, EvalError> {
    let n = as_number(value)?;
    if let Some(digits) = args.first() {
        let raw = as_number(digits)?;
        if !(0.0..=20.0).contains(&raw) || raw.fract() != 0.0 {
            return Err(EvalError::TypeMismatch("number pipe digits"));
        }
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "digits validated to 0..=20 integer"
        )]
        let d = raw as usize;
        return Ok(Value::Str(format!("{n:.d$}")));
    }
    Ok(Value::Str(format_num(n)))
}

#[allow(clippy::unnecessary_wraps)] // PipeFn signature is always Result
fn pipe_json(value: &Value, _: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Str(jsonish(value)))
}

fn jsonish(value: &Value) -> String {
    match value {
        Value::Str(s) => format!("\"{}\"", escape_json(s)),
        Value::Num(n) => format_num(*n),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => {
            let inner = items.iter().map(jsonish).collect::<Vec<_>>().join(",");
            format!("[{inner}]")
        }
        Value::Event(payload) => format!("\"{}\"", escape_json(&format!("{payload:?}"))),
        Value::Unit => "null".into(),
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_apply_case_and_json() {
        let reg = PipeRegistry::with_builtins();
        assert!(reg.contains("uppercase"));
        assert_eq!(
            reg.apply("uppercase", &Value::Str("ab".into()), &[])
                .unwrap(),
            Value::Str("AB".into())
        );
        assert_eq!(
            reg.apply("lowercase", &Value::Str("AB".into()), &[])
                .unwrap(),
            Value::Str("ab".into())
        );
        assert_eq!(
            reg.apply("json", &Value::Bool(true), &[]).unwrap(),
            Value::Str("true".into())
        );
        assert!(matches!(
            reg.apply("missing", &Value::Unit, &[]),
            Err(EvalError::UnknownPipe(_))
        ));
    }

    #[test]
    fn number_pipe_formats_digits() {
        let reg = PipeRegistry::with_builtins();
        assert_eq!(
            reg.apply("number", &Value::Num(1.5), &[Value::Num(2.0)])
                .unwrap(),
            Value::Str("1.50".into())
        );
        assert!(matches!(
            reg.apply("number", &Value::Str("x".into()), &[]),
            Err(EvalError::TypeMismatch(_))
        ));
    }
}
