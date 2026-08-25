mod ast;
mod banana;
mod component_io;
mod diag;
mod expr;
mod ir;
mod parser;
mod projection;
mod span;

pub use ast::{Attr, Element, ForBlock, IfBlock, NgTemplate, Node, Projection, Template};
pub use banana::{banana_event_name, banana_set_target, banana_write_expr, BANANA_SET_CALLEE};
pub use component_io::{builtin_tag_io, classify_bindings, TagIo};
pub use diag::{Diagnostic, Severity};
pub use expr::{BinOp, Expr, Literal, UnOp};
pub use ir::{
    event_handler_name, from_template as binding_ir, snapshot as binding_ir_snapshot, IrBinding,
    IrNode,
};
pub use parser::{parse, Parsed};
pub use projection::{
    collect_ng_templates, collect_projection_selects, has_default_projection, is_outlet_container,
    is_projection_tag, matches_select, select_param_name, template_outlet_ref,
};
pub use span::{line_col, Span};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Parsed {
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error())
    }

    #[must_use]
    pub fn ok(&self) -> bool {
        self.errors().next().is_none()
    }
}
