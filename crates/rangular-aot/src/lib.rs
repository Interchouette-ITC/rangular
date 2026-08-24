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
pub use rangular_parser::{binding_ir, binding_ir_snapshot, IrBinding, IrNode};

/// Structural binding IR for `source` (shared with runtime parity).
#[must_use]
pub fn structural_ir(source: &str, file: &str) -> Option<(Vec<IrNode>, String)> {
    let parsed = rangular_parser::parse(source, file);
    if !parsed.ok() {
        return None;
    }
    let nodes = binding_ir(&parsed.template);
    let snap = binding_ir_snapshot(&nodes);
    Some((nodes, snap))
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
