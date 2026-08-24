//! Minimal Host-side field checks (not Angular `NgModel` / forms).

use crate::value::Value;

/// Error when `value` is empty or whitespace-only.
#[must_use]
pub fn required(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        Some("This field is required")
    } else {
        None
    }
}

/// [`required`] against a [`Value`] (uses string / input-event text).
#[must_use]
pub fn required_value(value: &Value) -> Option<&'static str> {
    value
        .as_str()
        .map_or(Some("This field is required"), required)
}
