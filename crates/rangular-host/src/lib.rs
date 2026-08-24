#![forbid(unsafe_code)]

mod error;
mod event;
mod host;
mod validate;
mod value;

pub use error::HostError;
pub use event::EventPayload;
pub use host::Host;
pub use validate::{required, required_value};
pub use value::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
