use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use rangular_expr::{eval_with_pipes, Expr, Host, PipeRegistry, Value};
use rangular_host::EventPayload;
use send_wrapper::SendWrapper;

/// Shared host handle for AOT-generated views.
///
/// Wrapped for Leptos `Send` bounds; wasm CSR stays single-threaded.
pub struct HostCell<H: Host> {
    host: SendWrapper<Rc<RefCell<H>>>,
    pipes: Arc<PipeRegistry>,
}

impl<H: Host> Clone for HostCell<H> {
    fn clone(&self) -> Self {
        Self {
            host: SendWrapper::new(Rc::clone(&*self.host)),
            pipes: Arc::clone(&self.pipes),
        }
    }
}

impl<H: Host> HostCell<H> {
    #[must_use]
    pub fn new(host: H) -> Self {
        Self::with_pipes(host, PipeRegistry::builtins_arc())
    }

    #[must_use]
    pub fn with_pipes(host: H, pipes: Arc<PipeRegistry>) -> Self {
        Self {
            host: SendWrapper::new(Rc::new(RefCell::new(host))),
            pipes,
        }
    }

    #[must_use]
    pub fn eval_truthy(&self, expr: &Expr) -> bool {
        self.eval_scoped(expr, None, None).is_truthy()
    }

    #[must_use]
    pub fn eval_truthy_scoped(
        &self,
        expr: &Expr,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) -> bool {
        self.eval_scoped(expr, loop_name, loop_val).is_truthy()
    }

    #[must_use]
    pub fn eval_bool(&self, expr: &Expr) -> bool {
        match self.eval_value(expr) {
            Value::Bool(b) => b,
            v => v.is_truthy(),
        }
    }

    #[must_use]
    pub fn eval_bool_scoped(
        &self,
        expr: &Expr,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) -> bool {
        match self.eval_scoped(expr, loop_name, loop_val) {
            Value::Bool(b) => b,
            v => v.is_truthy(),
        }
    }

    #[must_use]
    pub fn eval_value(&self, expr: &Expr) -> Value {
        eval_with_pipes(expr, &mut *self.host.borrow_mut(), &self.pipes).unwrap_or(Value::Unit)
    }

    #[must_use]
    pub fn eval_scoped(
        &self,
        expr: &Expr,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) -> Value {
        if let Expr::Pipe { expr, name, args } = expr {
            let left = self.eval_scoped(expr, loop_name, loop_val);
            let arg_vals: Vec<Value> = args
                .iter()
                .map(|arg| self.eval_scoped(arg, loop_name, loop_val))
                .collect();
            return self
                .pipes
                .apply(name, &left, &arg_vals)
                .unwrap_or(Value::Unit);
        }
        if let (Some(name), Some(val)) = (loop_name, loop_val) {
            if matches!(expr, Expr::Ident(item) if item == name) {
                return Value::Str(val.to_owned());
            }
            if let Expr::Call { callee, args } = expr {
                if let Expr::Ident(callee_name) = callee.as_ref() {
                    let values: Vec<Value> = args
                        .iter()
                        .map(|arg| self.eval_scoped(arg, Some(name), Some(val)))
                        .collect();
                    if let Ok(v) = self.host.borrow_mut().call(callee_name, &values) {
                        return v;
                    }
                }
            }
        }
        self.eval_value(expr)
    }

    #[must_use]
    pub fn eval_list(&self, expr: &Expr) -> Vec<String> {
        match self.eval_value(expr) {
            Value::List(items) => items
                .into_iter()
                .filter_map(|item| match item {
                    Value::Str(s) => Some(s),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    #[must_use]
    pub fn prop_str(&self, expr: &Expr) -> String {
        value_display(&self.eval_value(expr))
    }

    #[must_use]
    pub fn prop_str_scoped(
        &self,
        expr: &Expr,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) -> String {
        value_display(&self.eval_scoped(expr, loop_name, loop_val))
    }

    pub fn emit_call(&self, name: &str, args: &[Value]) {
        let _ = self.host.borrow_mut().call(name, args);
    }

    pub fn emit_call_scoped(
        &self,
        name: &str,
        expr: &Expr,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) {
        let args = call_args_scoped(self, expr, loop_name, loop_val);
        self.emit_call(name, &args);
    }

    pub fn emit_event_call(&self, name: &str, expr: &Expr) {
        self.emit_call_scoped(name, expr, None, None);
    }

    pub fn emit_event_call_scoped(
        &self,
        name: &str,
        expr: &Expr,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) {
        self.emit_call_scoped(name, expr, loop_name, loop_val);
    }

    /// Set `$event` from a typed DOM payload, then invoke the handler expr.
    pub fn emit_dom_event_call(
        &self,
        name: &str,
        expr: &Expr,
        event_name: &str,
        event_value: String,
    ) {
        self.emit_dom_event_call_scoped(name, expr, event_name, event_value, None, None);
    }

    /// Like [`Self::emit_dom_event_call`] inside an `@for` item scope.
    pub fn emit_dom_event_call_scoped(
        &self,
        name: &str,
        expr: &Expr,
        event_name: &str,
        event_value: String,
        loop_name: Option<&str>,
        loop_val: Option<&str>,
    ) {
        if let Some(path) = rangular_parser::banana_set_target(expr) {
            let value = if event_name == "input" {
                Value::Str(event_value)
            } else {
                Value::from(EventPayload::from_dom(event_name, event_value))
            };
            let _ = self.host.borrow_mut().set(path, value);
            return;
        }
        let payload = EventPayload::from_dom(event_name, event_value);
        let _ = self.host.borrow_mut().set("$event", Value::from(payload));
        self.emit_call_scoped(name, expr, loop_name, loop_val);
    }
}

fn call_args_scoped<H: Host>(
    host: &HostCell<H>,
    expr: &Expr,
    loop_name: Option<&str>,
    loop_val: Option<&str>,
) -> Vec<Value> {
    match expr {
        Expr::Call { args, .. } => args
            .iter()
            .map(|arg| host.eval_scoped(arg, loop_name, loop_val))
            .collect(),
        _ => Vec::new(),
    }
}

fn value_display(value: &Value) -> String {
    match value {
        Value::Str(s) => s.clone(),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => items
            .iter()
            .map(value_display)
            .collect::<Vec<_>>()
            .join(","),
        Value::Event(payload) => match payload {
            EventPayload::Click => "event:click".into(),
            EventPayload::Input { value } => format!("event:input:{value}"),
            EventPayload::Error => "event:error".into(),
            EventPayload::Custom(inner) => format!("event:custom:{}", value_display(inner)),
        },
        Value::Unit => String::new(),
    }
}
