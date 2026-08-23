#![forbid(unsafe_code)]

mod error;
mod host;
mod value;

pub use error::HostError;
pub use host::Host;
pub use value::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
