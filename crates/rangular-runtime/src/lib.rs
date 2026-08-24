//! Runtime backend: interpret template AST against a host.

#![forbid(unsafe_code)]

mod error;
mod render;
mod snapshot;

pub use error::{RenderResult, RuntimeIssue};
pub use rangular_expr;
pub use rangular_host;
pub use rangular_parser;
pub use rangular_parser::{binding_ir, binding_ir_snapshot, IrBinding, IrNode};
pub use render::{interpret, render, VNode};
pub use snapshot::snapshot;

/// Structural binding IR for `source` (shared with AOT parity).
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
