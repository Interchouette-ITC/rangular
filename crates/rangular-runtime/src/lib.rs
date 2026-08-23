//! Runtime backend: interpret template AST against a host.

#![forbid(unsafe_code)]

mod error;
mod render;
mod snapshot;

pub use error::{RenderResult, RuntimeIssue};
pub use rangular_expr;
pub use rangular_host;
pub use rangular_parser;
pub use render::{interpret, render, VNode};
pub use snapshot::snapshot;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
