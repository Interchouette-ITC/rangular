#![forbid(unsafe_code)]

mod ast;
mod eval;
mod parse;
mod parse_issue;

pub use ast::{BinOp, Expr, Literal, UnOp};
pub use eval::{eval, EvalError};
pub use parse::parse;
pub use parse_issue::{IssueSeverity, ParseIssue, ParseResult};
pub use rangular_host::{Host, HostError, Value};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
