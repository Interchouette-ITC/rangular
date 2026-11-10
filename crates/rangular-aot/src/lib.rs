//! AOT backend: template AST to Leptos view factories.

#![forbid(unsafe_code)]

mod emit;
mod error;
mod expr_quote;
mod glue;
mod lower;
mod print;

pub use emit::{
    compile, compile_head, compile_head_named, compile_head_tokens, compile_head_tokens_named,
    compile_named, compile_tokens, compile_tokens_named,
};
pub use error::{AotIssue, EmitResult, EmitTokens};
pub use glue::HostCell;
pub use lower::{emit_rust, emit_rust_tokens};
pub use print::tokens_to_rust_source;
pub use rangular_expr;
pub use rangular_host;
pub use rangular_parser;
pub use rangular_parser::{IrBinding, IrNode, binding_ir, binding_ir_snapshot};

/// Structural binding IR for `source` (shared with runtime parity).
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

/// Escapes serialized JSON for use as the raw text of an HTML `<script>` element.
///
/// The input must already be valid JSON. Escaping `<` prevents a JSON string
/// containing `</script>` from terminating the enclosing HTML element. This
/// function does not validate or serialize JSON.
#[must_use]
pub fn escape_json_ld(input: &str) -> String {
    input.replace('<', "\\u003c")
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::escape_json_ld;

    #[test]
    fn json_ld_escaping_prevents_script_termination() {
        assert_eq!(
            escape_json_ld(r#"{"value":"</script>&>"}"#),
            r#"{"value":"\u003c/script>&>"}"#
        );
    }
}
