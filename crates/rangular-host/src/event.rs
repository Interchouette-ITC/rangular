//! DOM / component event payloads shared by AOT and runtime.

use crate::value::Value;

/// Typed event payload for `(click)`, `(input)`, `(error)`, `(load)`, and custom cases.
#[derive(Clone, Debug, PartialEq)]
pub enum EventPayload {
    Click { x: i32, y: i32 },
    Input { value: String },
    Error,
    Load,
    Custom(Box<Value>),
}

impl EventPayload {
    /// Build a payload from a DOM event name and optional string value (input target).
    #[must_use]
    pub fn from_dom(event_name: &str, event_value: String) -> Self {
        match event_name {
            "click" | "dblclick" | "auxclick" => {
                let (x, y) = click_xy(&event_value);
                Self::Click { x, y }
            }
            "input" | "change" => Self::Input { value: event_value },
            "error" => Self::Error,
            "load" => Self::Load,
            _ => Self::Custom(Box::new(Value::Str(event_value))),
        }
    }

    /// Human-readable label for demos and diagnostics.
    #[must_use]
    pub fn demo_label(&self) -> String {
        match self {
            Self::Click { x, y } => format!("Click {{ x: {x}, y: {y} }}"),
            Self::Input { value } => format!("Input {{ value: \"{value}\" }}"),
            Self::Error => "Error".into(),
            Self::Load => "Load".into(),
            Self::Custom(inner) => format!("Custom({inner:?})"),
        }
    }

    /// Stable label for snapshots / diagnostics.
    #[must_use]
    pub const fn kind_label(&self) -> &'static str {
        match self {
            Self::Click { .. } => "click",
            Self::Input { .. } => "input",
            Self::Error => "error",
            Self::Load => "load",
            Self::Custom(_) => "custom",
        }
    }
}

fn click_xy(raw: &str) -> (i32, i32) {
    let mut parts = raw.split(',');
    let x = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    let y = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    (x, y)
}

impl From<EventPayload> for Value {
    fn from(payload: EventPayload) -> Self {
        Self::Event(payload)
    }
}
