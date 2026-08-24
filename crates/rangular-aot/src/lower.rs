use std::collections::HashMap;

use proc_macro2::{Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote};
use rangular_parser::{
    collect_ng_templates, collect_projection_selects, has_default_projection, select_param_name,
    template_outlet_ref, Attr, Element, ForBlock, IfBlock, Node, Projection, Template,
};

use crate::error::{AotIssue, EmitResult, EmitTokens};
use crate::expr_quote::expr_tokens;

struct Scope<'a> {
    loop_item: Option<&'a str>,
}

impl Scope<'_> {
    const fn root() -> Scope<'static> {
        Scope { loop_item: None }
    }

    const fn with_loop_item(name: &str) -> Scope<'_> {
        Scope {
            loop_item: Some(name),
        }
    }

    fn scope_tokens(&self) -> TokenStream {
        self.loop_item.map_or_else(
            || quote! { None, None },
            |item| {
                let id = format_ident!("{item}");
                quote! { Some(#item), Some(#id.as_str()) }
            },
        )
    }
}

#[must_use]
pub fn emit_rust(template: &Template, fn_name: &str) -> EmitResult {
    let tokens = emit_rust_tokens(template, fn_name);
    EmitResult {
        code: crate::print::tokens_to_rust_source(&tokens.tokens),
        issues: tokens.issues,
    }
}

#[must_use]
pub fn emit_rust_tokens(template: &Template, fn_name: &str) -> EmitTokens {
    let mut issues = Vec::new();
    if template.nodes.is_empty() {
        issues.push(AotIssue::error("RANG401", "empty template"));
        return EmitTokens {
            tokens: proc_macro2::TokenStream::new(),
            issues,
        };
    }
    let templates: HashMap<String, Vec<Node>> =
        collect_ng_templates(&template.nodes).into_iter().collect();
    let scope = Scope::root();
    let Some(body) = lower_nodes(&template.nodes, &mut issues, &scope, &templates) else {
        return EmitTokens {
            tokens: proc_macro2::TokenStream::new(),
            issues,
        };
    };
    let ident = format_ident!("{fn_name}");
    let selects = collect_projection_selects(&template.nodes);
    let has_default = has_default_projection(&template.nodes);
    let tokens = if selects.is_empty() && has_projection(&template.nodes) {
        // Default-only `<ng-content>`: keep single `children` (AppRoot habit).
        quote! {
            #[allow(clippy::needless_pass_by_value, clippy::redundant_clone)]
            pub fn #ident<H: rangular_host::Host + 'static>(
                host: rangular_aot::HostCell<H>,
                children: Children,
            ) -> impl IntoView {
                let _ = &host;
                view! { #body }
            }
        }
    } else if !selects.is_empty() {
        let mut params = Vec::new();
        for select in &selects {
            let pname = format_ident!("{}", select_param_name(select));
            params.push(quote! { #pname: Children });
        }
        if has_default {
            params.push(quote! { children: Children });
        }
        quote! {
            #[allow(clippy::needless_pass_by_value, clippy::redundant_clone)]
            pub fn #ident<H: rangular_host::Host + 'static>(
                host: rangular_aot::HostCell<H>,
                #(#params),*
            ) -> impl IntoView {
                let _ = &host;
                view! { #body }
            }
        }
    } else {
        quote! {
            #[allow(clippy::needless_pass_by_value, clippy::redundant_clone)]
            pub fn #ident<H: rangular_host::Host + 'static>(
                host: rangular_aot::HostCell<H>,
            ) -> impl IntoView {
                let _ = &host;
                view! { #body }
            }
        }
    };
    EmitTokens { tokens, issues }
}

fn has_projection(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Projection(_) => true,
        Node::Element(el) => has_projection(&el.children),
        Node::NgTemplate(t) => has_projection(&t.body),
        Node::If(block) => {
            has_projection(&block.then_branch)
                || block
                    .else_branch
                    .as_ref()
                    .is_some_and(|nodes| has_projection(nodes))
        }
        Node::For(block) => has_projection(&block.body),
        Node::Text(_, _) | Node::Interpolation(_, _) | Node::Comment(_, _) => false,
    })
}

fn lower_nodes(
    nodes: &[Node],
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
) -> Option<TokenStream> {
    if nodes.is_empty() {
        return Some(quote! {});
    }
    let parts: Vec<_> = nodes
        .iter()
        .filter_map(|node| lower_node(node, issues, scope, templates))
        .collect();
    if parts.is_empty() {
        if nodes
            .iter()
            .all(|n| matches!(n, Node::Comment(_, _) | Node::NgTemplate(_)))
        {
            return Some(quote! {});
        }
        issues.push(AotIssue::error("RANG401", "no lowerable template nodes"));
        return None;
    }
    Some(quote! { #(#parts)* })
}

fn lower_node(
    node: &Node,
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
) -> Option<TokenStream> {
    match node {
        Node::Element(el) => lower_element(el, issues, scope, templates),
        Node::Text(text, _) => Some(quote! { #text }),
        Node::Interpolation(expr, _) => {
            let ex = expr_tokens(expr);
            let st = scope.scope_tokens();
            Some(quote! {{
                let host = host.clone();
                move || host.prop_str_scoped(&#ex, #st)
            }})
        }
        Node::Comment(_, _) | Node::NgTemplate(_) => None,
        Node::Projection(Projection { select, .. }) => select.as_ref().map_or_else(
            || Some(quote! { {children()} }),
            |sel| {
                let pname = format_ident!("{}", select_param_name(sel));
                Some(quote! { {#pname()} })
            },
        ),
        Node::If(block) => lower_if(block, issues, scope, templates),
        Node::For(block) => lower_for(block, issues, templates),
    }
}

fn lower_element(
    el: &Element,
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
) -> Option<TokenStream> {
    if let Some(name) = template_outlet_ref(&el.attrs) {
        let Some(body) = templates.get(name) else {
            issues.push(AotIssue::error(
                "RANG401",
                format!("unknown ngTemplateOutlet ref `{name}`"),
            ));
            return None;
        };
        return lower_nodes(body, issues, scope, templates);
    }
    if el.tag == "ng-container" {
        return lower_nodes(&el.children, issues, scope, templates);
    }
    let attrs = lower_attrs(&el.attrs, scope);
    let children = lower_children(&el.children, el.self_closing, issues, scope, templates)?;
    if el.tag.contains('-') {
        let tag_lit = el.tag.as_str();
        if el.self_closing {
            Some(quote! {
                <div data-rangular-component=#tag_lit #attrs />
            })
        } else {
            Some(quote! {
                <div data-rangular-component=#tag_lit #attrs>#children</div>
            })
        }
    } else {
        let tag = format_ident!("{}", sanitize_tag(&el.tag));
        if el.self_closing {
            Some(quote! { <#tag #attrs /> })
        } else {
            Some(quote! { <#tag #attrs>#children</#tag> })
        }
    }
}

fn lower_children(
    children: &[Node],
    self_closing: bool,
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
) -> Option<TokenStream> {
    if self_closing {
        return Some(quote! {});
    }
    lower_nodes(children, issues, scope, templates)
}

fn lower_attrs(attrs: &[Attr], scope: &Scope<'_>) -> TokenStream {
    let mut tokens = Vec::new();
    let st = scope.scope_tokens();
    for attr in attrs {
        if matches!(attr, Attr::Ref { .. }) {
            continue;
        }
        if matches!(
            attr,
            Attr::Property {
                name,
                ..
            } if name == "ngTemplateOutlet"
        ) {
            continue;
        }
        tokens.push(lower_one_attr(attr, &st));
    }
    if tokens.is_empty() {
        quote! {}
    } else {
        quote! { #(#tokens)* }
    }
}

fn lower_one_attr(attr: &Attr, st: &TokenStream) -> TokenStream {
    match attr {
        Attr::Static {
            name,
            value: Some(value),
            ..
        } => static_attr(name, value),
        Attr::Static {
            name, value: None, ..
        } => html_name(name),
        Attr::Property { name, expr, .. } if name == "disabled" => {
            let ex = expr_tokens(expr);
            prop_attr(
                name,
                &quote! {{
                    let host = host.clone();
                    move || host.eval_bool_scoped(&#ex, #st)
                }},
            )
        }
        Attr::Property { name, expr, .. } => {
            let ex = expr_tokens(expr);
            prop_attr(
                name,
                &quote! {{
                    let host = host.clone();
                    move || host.prop_str_scoped(&#ex, #st)
                }},
            )
        }
        Attr::Attribute { name, expr, .. } | Attr::Input { name, expr, .. } => {
            let attr_name = match attr {
                Attr::Input { name, .. } => format!("data-input-{name}"),
                _ => name.clone(),
            };
            let ex = expr_tokens(expr);
            attr_binding(
                &attr_name,
                &quote! {{
                    let host = host.clone();
                    move || host.prop_str_scoped(&#ex, #st)
                }},
            )
        }
        Attr::Class { name, expr, .. } => {
            let ex = expr_tokens(expr);
            class_attr(
                name,
                &quote! {{
                    let host = host.clone();
                    move || host.eval_truthy_scoped(&#ex, #st)
                }},
            )
        }
        Attr::Event { name, expr, .. } => lower_dom_event(name, expr, st),
        Attr::Output { name, expr, .. } => {
            let attr_name = format!("data-output-{name}");
            let handler = rangular_parser::event_handler_name(expr);
            static_attr(&attr_name, handler)
        }
        Attr::Ref { .. } => quote! {},
    }
}

fn lower_dom_event(name: &str, expr: &rangular_expr::Expr, st: &TokenStream) -> TokenStream {
    let handler = rangular_parser::event_handler_name(expr);
    let ex = expr_tokens(expr);
    event_attr(
        name,
        &quote! {{
            let host = host.clone();
            move |ev| {
                let event_value = {
                    use wasm_bindgen::JsCast;
                    ev.target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                        .map(|el| el.value())
                        .unwrap_or_default()
                };
                host.emit_dom_event_call_scoped(
                    #handler,
                    &#ex,
                    #name,
                    event_value,
                    #st,
                );
            }
        }},
    )
}

fn lower_if(
    block: &IfBlock,
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
) -> Option<TokenStream> {
    let cond = expr_tokens(&block.cond);
    let st = scope.scope_tokens();
    let then_view = lower_nodes(&block.then_branch, issues, scope, templates)?;
    if let Some(else_branch) = &block.else_branch {
        let else_view = lower_nodes(else_branch, issues, scope, templates)?;
        Some(quote! {{
            let host = host.clone();
            move || {
                if host.eval_truthy_scoped(&#cond, #st) {
                    view! { #then_view }.into_any()
                } else {
                    view! { #else_view }.into_any()
                }
            }
        }})
    } else {
        Some(quote! {
            <Show when={
                let host = host.clone();
                move || host.eval_truthy_scoped(&#cond, #st)
            }>
                #then_view
            </Show>
        })
    }
}

fn lower_for(
    block: &ForBlock,
    issues: &mut Vec<AotIssue>,
    templates: &HashMap<String, Vec<Node>>,
) -> Option<TokenStream> {
    let iter = expr_tokens(&block.iter);
    let item_name = block.item.as_str();
    let item_ident = format_ident!("{}", item_name);
    let scope = Scope::with_loop_item(item_name);
    let body = lower_nodes(&block.body, issues, &scope, templates)?;
    let st = scope.scope_tokens();
    let key = block.track.as_ref().map_or_else(
        || quote! { |#item_ident| #item_ident.clone() },
        |track| {
            let tr = expr_tokens(track);
            quote! {{
                let host = host.clone();
                move |#item_ident| host.prop_str_scoped(&#tr, #st)
            }}
        },
    );
    Some(quote! {
        <For
            each={
                let host = host.clone();
                move || host.eval_list(&#iter)
            }
            key=#key
            let:#item_ident
        >
            #body
        </For>
    })
}

fn html_name(name: &str) -> TokenStream {
    let mut out = TokenStream::new();
    for (i, part) in name.split('-').enumerate() {
        if i > 0 {
            let dash = Punct::new('-', Spacing::Joint);
            out.extend([TokenTree::Punct(dash)]);
        }
        if part.is_empty() {
            continue;
        }
        out.extend([TokenTree::Ident(format_ident!("{part}"))]);
    }
    out
}

fn static_attr(name: &str, value: &str) -> TokenStream {
    let mut out = html_name(name);
    let eq = Punct::new('=', Spacing::Joint);
    out.extend([TokenTree::Punct(eq)]);
    out.extend([TokenTree::Literal(Literal::string(value))]);
    out
}

fn prefix_attr(prefix: &str, name: &str, value: &TokenStream) -> TokenStream {
    let mut out = TokenStream::new();
    out.extend([TokenTree::Ident(format_ident!("{prefix}"))]);
    let colon = Punct::new(':', Spacing::Joint);
    out.extend([TokenTree::Punct(colon)]);
    out.extend(html_name(name));
    let eq = Punct::new('=', Spacing::Joint);
    out.extend([TokenTree::Punct(eq)]);
    out.extend(value.clone());
    out
}

fn prop_attr(name: &str, value: &TokenStream) -> TokenStream {
    prefix_attr("prop", name, value)
}

fn attr_binding(name: &str, value: &TokenStream) -> TokenStream {
    prefix_attr("attr", name, value)
}

fn class_attr(name: &str, value: &TokenStream) -> TokenStream {
    let mut out = TokenStream::new();
    out.extend([TokenTree::Ident(format_ident!("class"))]);
    let colon = Punct::new(':', Spacing::Joint);
    out.extend([TokenTree::Punct(colon)]);
    let paren = proc_macro2::Group::new(proc_macro2::Delimiter::Parenthesis, html_name(name));
    out.extend([TokenTree::Group(paren)]);
    let eq = Punct::new('=', Spacing::Joint);
    out.extend([TokenTree::Punct(eq)]);
    out.extend(value.clone());
    out
}

fn event_attr(name: &str, value: &TokenStream) -> TokenStream {
    prefix_attr("on", name, value)
}

const fn sanitize_tag(tag: &str) -> &str {
    if tag.is_empty() {
        "div"
    } else {
        tag
    }
}
