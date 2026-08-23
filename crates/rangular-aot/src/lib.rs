//! AOT backend: template AST to Leptos view factories.

#![forbid(unsafe_code)]

mod emit;
mod error;
mod expr_quote;
mod glue;
mod lower;
mod print;

pub use emit::{compile, compile_named, compile_tokens, compile_tokens_named};
pub use error::{AotIssue, EmitResult, EmitTokens};
pub use glue::HostCell;
pub use lower::{emit_rust, emit_rust_tokens};
pub use print::tokens_to_rust_source;
pub use rangular_expr;
pub use rangular_host;
pub use rangular_parser;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
