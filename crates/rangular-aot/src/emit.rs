use rangular_parser::{builtin_tag_io, classify_bindings, parse, Diagnostic};

use crate::error::{AotIssue, EmitResult, EmitTokens};
use crate::lower::{emit_rust, emit_rust_tokens};

#[must_use]
pub fn compile(source: &str, fn_name: &str) -> EmitResult {
    compile_named(source, "<template>", fn_name)
}

#[must_use]
pub fn compile_named(source: &str, file: &str, fn_name: &str) -> EmitResult {
    let mut parsed = parse(source, file);
    let mut issues: Vec<AotIssue> = parsed
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(diag_issue)
        .collect();
    if !issues.is_empty() {
        return EmitResult {
            code: String::new(),
            issues,
        };
    }
    classify_bindings(&mut parsed.template, &builtin_tag_io());
    let mut out = emit_rust(&parsed.template, fn_name);
    issues.append(&mut out.issues);
    EmitResult {
        code: out.code,
        issues,
    }
}

#[must_use]
pub fn compile_tokens(source: &str, fn_name: &str) -> EmitTokens {
    compile_tokens_named(source, "<template>", fn_name)
}

#[must_use]
pub fn compile_tokens_named(source: &str, file: &str, fn_name: &str) -> EmitTokens {
    let mut parsed = parse(source, file);
    let issues: Vec<AotIssue> = parsed
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(diag_issue)
        .collect();
    if !issues.is_empty() {
        return EmitTokens {
            tokens: proc_macro2::TokenStream::new(),
            issues,
        };
    }
    classify_bindings(&mut parsed.template, &builtin_tag_io());
    emit_rust_tokens(&parsed.template, fn_name)
}

fn diag_issue(d: &Diagnostic) -> AotIssue {
    AotIssue::error(d.code, d.message.clone())
}
