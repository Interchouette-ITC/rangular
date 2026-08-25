//! Format `TokenStream` as Rust source without using span-backed `Display`.
//!
//! Used by AOT string emit so consumers can `include!` generated view code.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

#[must_use]
pub fn tokens_to_rust_source(tokens: &TokenStream) -> String {
    let mut out = String::new();
    let mut needs_space = false;
    let mut last_char = '\0';
    write_stream(tokens, &mut out, &mut needs_space, &mut last_char);
    out
}

fn write_stream(
    tokens: &TokenStream,
    out: &mut String,
    needs_space: &mut bool,
    last_char: &mut char,
) {
    for tt in tokens.clone() {
        write_tree(tt, out, needs_space, last_char);
    }
}

fn write_tree(tt: TokenTree, out: &mut String, needs_space: &mut bool, last_char: &mut char) {
    match tt {
        TokenTree::Group(group) => {
            let (open, close) = match group.delimiter() {
                Delimiter::Parenthesis => ('(', ')'),
                Delimiter::Brace => ('{', '}'),
                Delimiter::Bracket => ('[', ']'),
                Delimiter::None => {
                    write_stream(&group.stream(), out, needs_space, last_char);
                    return;
                }
            };
            if *needs_space {
                out.push(' ');
            }
            out.push(open);
            *last_char = open;
            let mut inner_space = false;
            let mut inner_last = '\0';
            write_stream(&group.stream(), out, &mut inner_space, &mut inner_last);
            out.push(close);
            *last_char = close;
            *needs_space = true;
        }
        TokenTree::Ident(ident) => {
            if *needs_space {
                out.push(' ');
            }
            let s = ident.to_string();
            out.push_str(&s);
            *last_char = s.chars().last().unwrap_or('\0');
            *needs_space = true;
        }
        TokenTree::Literal(lit) => {
            if *needs_space {
                out.push(' ');
            }
            let s = lit.to_string();
            out.push_str(&s);
            *last_char = s.chars().last().unwrap_or('\0');
            *needs_space = true;
        }
        TokenTree::Punct(punct) => {
            let ch = punct.as_char();
            out.push(ch);
            *needs_space = match ch {
                '<' | ':' | '.' | '#' | '=' => false,
                '/' if *last_char == '<' => false,
                _ => punct.spacing() == Spacing::Alone,
            };
            *last_char = ch;
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::{Literal, Punct, Spacing, TokenTree};
    use quote::format_ident;

    use super::tokens_to_rust_source;

    #[test]
    fn joint_hyphen_and_eq_have_no_spaces() {
        let mut attr = proc_macro2::TokenStream::new();
        attr.extend([TokenTree::Ident(format_ident!("aria"))]);
        attr.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
        attr.extend([TokenTree::Ident(format_ident!("label"))]);
        attr.extend([TokenTree::Punct(Punct::new('=', Spacing::Joint))]);
        attr.extend([TokenTree::Literal(Literal::string("Seed"))]);
        let src = tokens_to_rust_source(&attr);
        assert_eq!(src, r#"aria-label="Seed""#);
    }

    #[test]
    fn class_attr_joint_eq_before_brace() {
        use proc_macro2::{Delimiter, TokenStream};
        use quote::quote;

        let value: TokenStream = quote! {{ false }};
        let mut out = proc_macro2::TokenStream::new();
        out.extend([TokenTree::Ident(format_ident!("class"))]);
        out.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
        let mut name = proc_macro2::TokenStream::new();
        name.extend([TokenTree::Ident(format_ident!("color"))]);
        name.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
        name.extend([TokenTree::Ident(format_ident!("field__toggle"))]);
        name.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
        name.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
        name.extend([TokenTree::Ident(format_ident!("open"))]);
        let paren = proc_macro2::Group::new(Delimiter::Parenthesis, name);
        out.extend([TokenTree::Group(paren)]);
        out.extend([TokenTree::Punct(Punct::new('=', Spacing::Joint))]);
        out.extend(value);
        let src = tokens_to_rust_source(&out);
        assert!(src.contains("class:(color-field__toggle--open)={"), "{src}");
    }

    #[test]
    fn static_attr_helper_shape() {
        let mut out = proc_macro2::TokenStream::new();
        out.extend([TokenTree::Ident(format_ident!("class"))]);
        out.extend([TokenTree::Punct(Punct::new('=', Spacing::Joint))]);
        out.extend([TokenTree::Literal(Literal::string("seed-bar"))]);
        let src = tokens_to_rust_source(&out);
        assert_eq!(src, r#"class="seed-bar""#);
    }
}
