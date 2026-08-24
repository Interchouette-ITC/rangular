//! Runtime backend: interpret template AST against a host.

#![forbid(unsafe_code)]

mod error;
mod render;
mod slots;
mod snapshot;

pub use error::{RenderResult, RuntimeIssue};
pub use rangular_expr;
pub use rangular_host;
pub use rangular_parser;
pub use rangular_parser::{binding_ir, binding_ir_snapshot, IrBinding, IrNode};
pub use render::{
    interpret, interpret_with_pipes, interpret_with_slot, interpret_with_slot_and_pipes,
    interpret_with_slots, interpret_with_slots_and_pipes, render, render_with_slot,
    render_with_slot_and_pipes, render_with_slots, render_with_slots_and_pipes,
};
pub use slots::{ProjectionBag, VNode};
pub use snapshot::snapshot;

/// Structural binding IR for `source` (shared with AOT parity).
#[must_use]
pub fn structural_ir(source: &str, file: &str) -> Option<(Vec<IrNode>, String)> {
    let mut parsed = rangular_parser::parse(source, file);
    if !parsed.ok() {
        return None;
    }
    rangular_parser::classify_bindings(&mut parsed.template, &rangular_parser::builtin_tag_io());
    let nodes = binding_ir(&parsed.template);
    let snap = binding_ir_snapshot(&nodes);
    Some((nodes, snap))
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
