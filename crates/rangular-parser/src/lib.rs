mod ast;
mod diag;
mod expr;
mod ir;
mod parser;
mod span;

pub use ast::{Attr, Element, ForBlock, IfBlock, Node, Template};
pub use diag::{Diagnostic, Severity};
pub use expr::{BinOp, Expr, Literal, UnOp};
pub use ir::{
    event_handler_name, from_template as binding_ir, snapshot as binding_ir_snapshot, IrBinding,
    IrNode,
};
pub use parser::{parse, Parsed};
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
