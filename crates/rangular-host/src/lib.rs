#![forbid(unsafe_code)]

mod error;
mod event;
mod for_scope;
mod host;
mod validate;
mod value;

pub use error::HostError;
pub use event::EventPayload;
pub use for_scope::{for_implicit_value, LoopScope};
pub use host::Host;
pub use regex::Regex;
pub use validate::{
    max_length, max_length_value, min_length, min_length_value, pattern, pattern_value, required,
    required_value, show_when_dirty,
};
pub use value::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
