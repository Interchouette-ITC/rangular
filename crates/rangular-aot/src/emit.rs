use rangular_parser::{Diagnostic, builtin_tag_io, classify_bindings, parse};

use crate::error::{AotIssue, EmitResult, EmitTokens};
use crate::lower::{emit_rust_tokens, emit_rust_tokens_head};

#[must_use]
pub fn compile(source: &str, fn_name: &str) -> EmitResult {
    compile_named(source, "<template>", fn_name)
}

#[must_use]
pub fn compile_named(source: &str, file: &str, fn_name: &str) -> EmitResult {
    compile_named_with_mode(source, file, fn_name, false)
}

/// Compiles a Rangular document-head template with head-specific validation.
#[must_use]
pub fn compile_head(source: &str, fn_name: &str) -> EmitResult {
    compile_head_named(source, "<head-template>", fn_name)
}

/// Compiles a named Rangular document-head template with head-specific validation.
#[must_use]
pub fn compile_head_named(source: &str, file: &str, fn_name: &str) -> EmitResult {
    compile_named_with_mode(source, file, fn_name, true)
}

fn compile_named_with_mode(source: &str, file: &str, fn_name: &str, head_mode: bool) -> EmitResult {
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
    let mut out = if head_mode {
        emit_rust_tokens_head(&parsed.template, fn_name)
    } else {
        emit_rust_tokens(&parsed.template, fn_name)
    };
    let code = crate::print::tokens_to_rust_source(&out.tokens);
    issues.append(&mut out.issues);
    EmitResult { code, issues }
}

#[must_use]
pub fn compile_tokens(source: &str, fn_name: &str) -> EmitTokens {
    compile_tokens_named(source, "<template>", fn_name)
}

#[must_use]
pub fn compile_tokens_named(source: &str, file: &str, fn_name: &str) -> EmitTokens {
    compile_tokens_named_with_mode(source, file, fn_name, false)
}

#[must_use]
pub fn compile_head_tokens(source: &str, fn_name: &str) -> EmitTokens {
    compile_head_tokens_named(source, "<head-template>", fn_name)
}

/// Compiles a named document-head template directly to Rust tokens.
#[must_use]
pub fn compile_head_tokens_named(source: &str, file: &str, fn_name: &str) -> EmitTokens {
    compile_tokens_named_with_mode(source, file, fn_name, true)
}

fn compile_tokens_named_with_mode(
    source: &str,
    file: &str,
    fn_name: &str,
    head_mode: bool,
) -> EmitTokens {
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
    if head_mode {
        emit_rust_tokens_head(&parsed.template, fn_name)
    } else {
        emit_rust_tokens(&parsed.template, fn_name)
    }
}

fn diag_issue(d: &Diagnostic) -> AotIssue {
    AotIssue::error(d.code, d.message.clone())
}
