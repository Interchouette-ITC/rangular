#![forbid(unsafe_code)]

mod ast;
mod eval;
mod parse;
mod parse_issue;
mod pipe;

pub use ast::{BinOp, Expr, Literal, UnOp};
pub use eval::{EvalError, eval, eval_with_pipes};
pub use parse::parse;
pub use parse_issue::{IssueSeverity, ParseIssue, ParseResult};
pub use pipe::{PipeFn, PipeRegistry};
pub use rangular_host::{Host, HostError, Value};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
