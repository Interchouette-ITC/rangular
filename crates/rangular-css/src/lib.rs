//! Component SCSS: compile, `:host` rewrite, and emulated encapsulation.

#![forbid(unsafe_code)]

mod encapsulate;
mod error;
mod globals;

pub use encapsulate::{encapsulate, encapsulate_css, ScopeAttrs};
pub use error::{CssIssue, CssResult};
pub use globals::is_global_selector;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
