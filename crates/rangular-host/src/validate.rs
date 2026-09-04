//! Host-side field checks (not Angular `NgModel` / forms).
//!
//! Prefer compiling a [`Regex`] once when the Host is constructed; pass `&Regex`
//! into [`pattern`] / [`pattern_value`] on each `get` rather than recompiling.

use regex::Regex;

use crate::value::Value;

/// Show validation UI only after the user has edited the field (Angular dirty semantics).
#[must_use]
pub const fn show_when_dirty(invalid: bool, dirty: bool) -> bool {
    invalid && dirty
}

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

/// Error when non-empty `value` has fewer than `min` Unicode scalar values.
///
/// Empty strings pass so optional fields can use this without [`required`].
#[must_use]
pub fn min_length(value: &str, min: usize) -> Option<&'static str> {
    let len = value.chars().count();
    if len == 0 || len >= min {
        None
    } else {
        Some("Minimum length is not met")
    }
}

/// [`min_length`] against a [`Value`] (missing / non-string treated as empty).
#[must_use]
pub fn min_length_value(value: &Value, min: usize) -> Option<&'static str> {
    min_length(value.as_str().unwrap_or(""), min)
}

/// Error when `value` has more than `max` Unicode scalar values.
#[must_use]
pub fn max_length(value: &str, max: usize) -> Option<&'static str> {
    if value.chars().count() <= max {
        None
    } else {
        Some("Maximum length exceeded")
    }
}

/// [`max_length`] against a [`Value`] (missing / non-string treated as empty).
#[must_use]
pub fn max_length_value(value: &Value, max: usize) -> Option<&'static str> {
    max_length(value.as_str().unwrap_or(""), max)
}

/// Error when `value` does not fully match `re` ([`Regex::is_match`]).
#[must_use]
pub fn pattern(value: &str, re: &Regex) -> Option<&'static str> {
    if re.is_match(value) {
        None
    } else {
        Some("Invalid format")
    }
}

/// [`pattern`] against a [`Value`] (missing / non-string treated as empty).
#[must_use]
pub fn pattern_value(value: &Value, re: &Regex) -> Option<&'static str> {
    pattern(value.as_str().unwrap_or(""), re)
}
