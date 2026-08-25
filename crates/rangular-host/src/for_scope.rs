//! `@for` loop scope: item alias and Angular implicit variables.

use crate::value::Value;

/// Active `@for` iteration when evaluating expressions in the loop body.
#[derive(Clone, Copy, Default)]
pub struct LoopScope<'a> {
    pub item_name: Option<&'a str>,
    pub item_val: Option<&'a str>,
    pub index: Option<usize>,
    pub count: Option<usize>,
}

impl LoopScope<'static> {
    pub const NONE: Self = Self {
        item_name: None,
        item_val: None,
        index: None,
        count: None,
    };
}

fn loop_usize(value: usize) -> f64 {
    u32::try_from(value).map_or_else(|_| f64::from(u32::MAX), f64::from)
}

/// Resolve Angular-shaped `@for` implicit identifiers (`$index`, `$first`, …).
#[must_use]
pub fn for_implicit_value(name: &str, index: usize, count: usize) -> Option<Value> {
    match name {
        "$index" => Some(Value::Num(loop_usize(index))),
        "$count" => Some(Value::Num(loop_usize(count))),
        "$first" => Some(Value::Bool(index == 0)),
        "$last" => Some(Value::Bool(count > 0 && index + 1 == count)),
        "$even" => Some(Value::Bool(index.is_multiple_of(2))),
        "$odd" => Some(Value::Bool(!index.is_multiple_of(2))),
        _ => None,
    }
}
