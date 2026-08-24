//! DOM / component event payloads shared by AOT and runtime.

use crate::value::Value;

/// Typed event payload for `(click)`, `(input)`, `(error)`, and custom cases.
#[derive(Clone, Debug, PartialEq)]
pub enum EventPayload {
    Click,
    Input { value: String },
    Error,
    Custom(Box<Value>),
}

impl EventPayload {
    /// Build a payload from a DOM event name and optional string value (input target).
    #[must_use]
    pub fn from_dom(event_name: &str, event_value: String) -> Self {
        match event_name {
            "click" => Self::Click,
            "input" | "change" => Self::Input { value: event_value },
            "error" => Self::Error,
            _ => Self::Custom(Box::new(Value::Str(event_value))),
        }
    }

    /// Stable label for snapshots / diagnostics.
    #[must_use]
    pub const fn kind_label(&self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::Input { .. } => "input",
            Self::Error => "error",
            Self::Custom(_) => "custom",
        }
    }
}

impl From<EventPayload> for Value {
    fn from(payload: EventPayload) -> Self {
        Self::Event(payload)
    }
}
