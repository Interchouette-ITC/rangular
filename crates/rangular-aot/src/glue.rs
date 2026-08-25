use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use rangular_expr::{eval_with_pipes, Expr, Host, PipeRegistry, Value};
use rangular_host::{for_implicit_value, EventPayload, LoopScope};
use send_wrapper::SendWrapper;

/// Shared host handle for AOT-generated views.
///
/// Wrapped for Leptos `Send` bounds; wasm CSR stays single-threaded.
pub struct HostCell<H: Host> {
    host: SendWrapper<Rc<RefCell<H>>>,
    pipes: Arc<PipeRegistry>,
    /// Last DOM `$event` payload for handler calls such as `onInput($event)`.
    event: SendWrapper<Rc<RefCell<Value>>>,
}

impl<H: Host> Clone for HostCell<H> {
    fn clone(&self) -> Self {
        Self {
            host: SendWrapper::new(Rc::clone(&*self.host)),
            pipes: Arc::clone(&self.pipes),
            event: SendWrapper::new(Rc::clone(&*self.event)),
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
            event: SendWrapper::new(Rc::new(RefCell::new(Value::Unit))),
        }
    }

    #[must_use]
    pub fn eval_truthy(&self, expr: &Expr) -> bool {
        self.eval_scoped(expr, LoopScope::NONE).is_truthy()
    }

    #[must_use]
    pub fn eval_truthy_scoped(&self, expr: &Expr, loop_scope: LoopScope<'_>) -> bool {
        self.eval_scoped(expr, loop_scope).is_truthy()
    }

    #[must_use]
    pub fn eval_bool(&self, expr: &Expr) -> bool {
        match self.eval_value(expr) {
            Value::Bool(b) => b,
            v => v.is_truthy(),
        }
    }

    #[must_use]
    pub fn eval_bool_scoped(&self, expr: &Expr, loop_scope: LoopScope<'_>) -> bool {
        match self.eval_scoped(expr, loop_scope) {
            Value::Bool(b) => b,
            v => v.is_truthy(),
        }
    }

    #[must_use]
    pub fn eval_value(&self, expr: &Expr) -> Value {
        if let Expr::Ident(name) = expr {
            if name == "$event" {
                return self.event.borrow().clone();
            }
        }
        eval_with_pipes(expr, &mut *self.host.borrow_mut(), &self.pipes).unwrap_or(Value::Unit)
    }

    #[must_use]
    pub fn eval_scoped(&self, expr: &Expr, loop_scope: LoopScope<'_>) -> Value {
        if let Expr::Pipe { expr, name, args } = expr {
            let left = self.eval_scoped(expr, loop_scope);
            let arg_vals: Vec<Value> = args
                .iter()
                .map(|arg| self.eval_scoped(arg, loop_scope))
                .collect();
            return self
                .pipes
                .apply(name, &left, &arg_vals)
                .unwrap_or(Value::Unit);
        }
        if let Expr::Ident(name) = expr {
            if let Some(item_name) = loop_scope.item_name {
                if name == item_name {
                    if let Some(val) = loop_scope.item_val {
                        return Value::Str(val.to_owned());
                    }
                }
            }
            if let (Some(index), Some(count)) = (loop_scope.index, loop_scope.count) {
                if let Some(value) = for_implicit_value(name, index, count) {
                    return value;
                }
            }
        }
        if let Expr::Call { callee, args } = expr {
            if let Expr::Ident(callee_name) = callee.as_ref() {
                let values: Vec<Value> = args
                    .iter()
                    .map(|arg| self.eval_scoped(arg, loop_scope))
                    .collect();
                if let Ok(v) = self.host.borrow_mut().call(callee_name, &values) {
                    return v;
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
    pub fn prop_str_scoped(&self, expr: &Expr, loop_scope: LoopScope<'_>) -> String {
        value_display(&self.eval_scoped(expr, loop_scope))
    }

    pub fn emit_call(&self, name: &str, args: &[Value]) {
        let _ = self.host.borrow_mut().call(name, args);
    }

    pub fn emit_call_scoped(&self, name: &str, expr: &Expr, loop_scope: LoopScope<'_>) {
        let args = call_args_scoped(self, expr, loop_scope);
        self.emit_call(name, &args);
    }

    pub fn emit_event_call(&self, name: &str, expr: &Expr) {
        self.emit_call_scoped(name, expr, LoopScope::NONE);
    }

    pub fn emit_event_call_scoped(&self, name: &str, expr: &Expr, loop_scope: LoopScope<'_>) {
        self.emit_call_scoped(name, expr, loop_scope);
    }

    /// Set `$event` from a typed DOM payload, then invoke the handler expr.
    pub fn emit_dom_event_call(
        &self,
        name: &str,
        expr: &Expr,
        event_name: &str,
        event_value: String,
    ) {
        self.emit_dom_event_call_scoped(name, expr, event_name, event_value, LoopScope::NONE);
    }

    /// Like [`Self::emit_dom_event_call`] inside an `@for` item scope.
    pub fn emit_dom_event_call_scoped(
        &self,
        name: &str,
        expr: &Expr,
        event_name: &str,
        event_value: String,
        loop_scope: LoopScope<'_>,
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
        let event = Value::from(payload);
        *self.event.borrow_mut() = event.clone();
        let _ = self.host.borrow_mut().set("$event", event);
        self.emit_call_scoped(name, expr, loop_scope);
    }
}

fn call_args_scoped<H: Host>(
    host: &HostCell<H>,
    expr: &Expr,
    loop_scope: LoopScope<'_>,
) -> Vec<Value> {
    match expr {
        Expr::Call { args, .. } => args
            .iter()
            .map(|arg| host.eval_scoped(arg, loop_scope))
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
            EventPayload::Click { x, y } => format!("event:click:{x},{y}"),
            EventPayload::Input { value } => format!("event:input:{value}"),
            EventPayload::Error => "event:error".into(),
            EventPayload::Load => "event:load".into(),
            EventPayload::Custom(inner) => format!("event:custom:{}", value_display(inner)),
        },
        Value::Unit => String::new(),
    }
}
