#![forbid(unsafe_code)]

mod error;
mod event;
mod host;
mod value;

pub use error::HostError;
pub use event::EventPayload;
pub use host::Host;
pub use value::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
