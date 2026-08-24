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
pub use validate::{required, required_value, show_when_dirty};
pub use value::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
