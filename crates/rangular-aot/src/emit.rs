use rangular_parser::{Diagnostic, builtin_tag_io, classify_bindings, parse, parse_head_template};

use crate::error::{AotIssue, EmitResult, EmitTokens};
use crate::lower::{
    HeadComponentPaths, emit_rust_tokens, emit_rust_tokens_head, emit_rust_tokens_head_with_adapter,
};

/// Leptos component paths used to opt a generated head view into a document
/// integration's title, metadata, and link lifecycle.
///
/// The components receive `text` for titles, `head_key` for keyed entries, and
/// standard head attributes as props. The compiler does not depend on any
/// particular component library; callers provide paths from their integration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadComponentAdapter {
    title: String,
    meta: String,
    link: String,
}

impl HeadComponentAdapter {
    /// Creates an adapter from three Rust component paths.
    #[must_use]
    pub fn new(title: impl Into<String>, meta: impl Into<String>, link: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            meta: meta.into(),
            link: link.into(),
        }
    }

    fn paths(&self) -> Result<HeadComponentPaths, AotIssue> {
        Ok(HeadComponentPaths {
            title: parse_component_path("title", &self.title)?,
            meta: parse_component_path("meta", &self.meta)?,
            link: parse_component_path("link", &self.link)?,
        })
    }
}

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

/// Compiles a named head template to caller-supplied Leptos lifecycle components.
///
/// In this mode, `head-key="..."` is consumed as a stable registry key on
/// `<title>`, `<meta>`, and `<link>` elements. Without an adapter,
/// [`compile_head_named`] continues to emit native Leptos HTML elements.
#[must_use]
pub fn compile_head_named_with_adapter(
    source: &str,
    file: &str,
    fn_name: &str,
    adapter: &HeadComponentAdapter,
) -> EmitResult {
    let adapter = match adapter.paths() {
        Ok(adapter) => adapter,
        Err(issue) => {
            return EmitResult {
                code: String::new(),
                issues: vec![issue],
            };
        }
    };
    compile_named_with_options(source, file, fn_name, true, Some(adapter))
}

/// Compiles a head template to caller-supplied Leptos lifecycle components.
#[must_use]
pub fn compile_head_with_adapter(
    source: &str,
    fn_name: &str,
    adapter: &HeadComponentAdapter,
) -> EmitResult {
    compile_head_named_with_adapter(source, "<head-template>", fn_name, adapter)
}

/// Compiles a head template to caller-supplied Leptos lifecycle components as Rust tokens.
#[must_use]
pub fn compile_head_tokens_with_adapter(
    source: &str,
    fn_name: &str,
    adapter: &HeadComponentAdapter,
) -> EmitTokens {
    compile_head_tokens_named_with_adapter(source, "<head-template>", fn_name, adapter)
}

/// Compiles a named head template to caller-supplied Leptos lifecycle components as Rust tokens.
#[must_use]
pub fn compile_head_tokens_named_with_adapter(
    source: &str,
    file: &str,
    fn_name: &str,
    adapter: &HeadComponentAdapter,
) -> EmitTokens {
    let adapter = match adapter.paths() {
        Ok(adapter) => adapter,
        Err(issue) => {
            return EmitTokens {
                tokens: proc_macro2::TokenStream::new(),
                issues: vec![issue],
            };
        }
    };
    compile_tokens_named_with_options(source, file, fn_name, true, Some(adapter))
}

fn compile_named_with_mode(source: &str, file: &str, fn_name: &str, head_mode: bool) -> EmitResult {
    compile_named_with_options(source, file, fn_name, head_mode, None)
}

fn compile_named_with_options(
    source: &str,
    file: &str,
    fn_name: &str,
    head_mode: bool,
    head_adapter: Option<HeadComponentPaths>,
) -> EmitResult {
    let mut parsed = if head_mode {
        parse_head_template(source, file)
    } else {
        parse(source, file)
    };
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
        if let Some(adapter) = head_adapter {
            emit_rust_tokens_head_with_adapter(&parsed.template, fn_name, adapter)
        } else {
            emit_rust_tokens_head(&parsed.template, fn_name)
        }
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
    compile_tokens_named_with_options(source, file, fn_name, head_mode, None)
}

fn compile_tokens_named_with_options(
    source: &str,
    file: &str,
    fn_name: &str,
    head_mode: bool,
    head_adapter: Option<HeadComponentPaths>,
) -> EmitTokens {
    let mut parsed = if head_mode {
        parse_head_template(source, file)
    } else {
        parse(source, file)
    };
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
        if let Some(adapter) = head_adapter {
            emit_rust_tokens_head_with_adapter(&parsed.template, fn_name, adapter)
        } else {
            emit_rust_tokens_head(&parsed.template, fn_name)
        }
    } else {
        emit_rust_tokens(&parsed.template, fn_name)
    }
}

fn parse_component_path(kind: &str, path: &str) -> Result<proc_macro2::TokenStream, AotIssue> {
    let tokens = path.parse::<proc_macro2::TokenStream>().map_err(|error| {
        AotIssue::error(
            "RANG404",
            format!("invalid {kind} head component path `{path}`: {error}"),
        )
    })?;
    if tokens.is_empty() {
        return Err(AotIssue::error(
            "RANG404",
            format!("the {kind} head component path cannot be empty"),
        ));
    }
    Ok(tokens)
}

fn diag_issue(d: &Diagnostic) -> AotIssue {
    AotIssue::error(d.code, d.message.clone())
}
