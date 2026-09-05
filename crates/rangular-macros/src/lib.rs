//! Proc macros for rangular component templates.
//!
//! Expands to `include!` of AOT output under `OUT_DIR/rangular/`. The consuming
//! crate's `build.rs` must call `rangular_aot::compile_named` and write
//! `{fn_name}.rs` there before compiling the crate that invokes this macro.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn rangular_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TemplateInput);
    let fn_name = input
        .fn_name
        .unwrap_or_else(|| default_fn_name(&input.path));
    let suffix = format!("/rangular/{fn_name}.rs");
    let suffix_lit = LitStr::new(&suffix, proc_macro2::Span::call_site());
    quote! {
        include!(concat!(env!("OUT_DIR"), #suffix_lit));
    }
    .into()
}

struct TemplateInput {
    path: String,
    fn_name: Option<String>,
}

impl syn::parse::Parse for TemplateInput {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let path_lit: LitStr = input.parse()?;
        let path = path_lit.value();
        let fn_name = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let name_lit: LitStr = input.parse()?;
            Some(name_lit.value())
        } else {
            None
        };
        Ok(Self { path, fn_name })
    }
}

fn default_fn_name(path: &str) -> String {
    PathBuf::from(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("template")
        .replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::{default_fn_name, TemplateInput};
    use syn::parse_str;

    #[test]
    fn default_fn_name_replaces_hyphens() {
        assert_eq!(default_fn_name("seed-bar.html"), "seed_bar");
    }

    #[test]
    fn default_fn_name_fallbacks() {
        assert_eq!(default_fn_name("plain.html"), "plain");
        assert_eq!(default_fn_name(""), "template");
        assert_eq!(default_fn_name("/tmp/foo-bar.ng.html"), "foo_bar.ng");
    }

    #[test]
    fn template_input_parses_path_only_and_with_name() {
        let path_only: TemplateInput = parse_str(r#""seed-bar.html""#).expect("path");
        assert_eq!(path_only.path, "seed-bar.html");
        assert!(path_only.fn_name.is_none());

        let named: TemplateInput = parse_str(r#""seed-bar.html", "seed_bar_view""#).expect("named");
        assert_eq!(named.path, "seed-bar.html");
        assert_eq!(named.fn_name.as_deref(), Some("seed_bar_view"));
    }
}
