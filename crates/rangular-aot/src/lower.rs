use std::collections::HashMap;

use proc_macro2::{Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote};
use rangular_expr::Expr;
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
}

fn scope_args(scope: &Scope<'_>) -> TokenStream {
    scope.loop_item.map_or_else(
        || quote! { None, None },
        |item| quote! { Some(#item), Some(__rangular_loop.as_str()) },
    )
}

fn scoped_closure_body(scope: &Scope<'_>, invoke: &TokenStream) -> TokenStream {
    scope.loop_item.map_or_else(
        || quote! { move || #invoke },
        |_item| {
            quote! {
                move || {
                    let __rangular_loop = std::sync::Arc::clone(&__rangular_loop_store.get_value());
                    #invoke
                }
            }
        },
    )
}

fn hoist_host_closure(
    hoist: &mut HoistState,
    scope: &Scope<'_>,
    invoke: &TokenStream,
) -> TokenStream {
    let invoke = scoped_closure_body(scope, invoke);
    hoist.hoist_closure(&invoke)
}

fn hoist_event_closure(
    hoist: &mut HoistState,
    scope: &Scope<'_>,
    event_name: &str,
    ev_ty: &TokenStream,
    body: &TokenStream,
) -> TokenStream {
    let ev = if event_name == "error" {
        format_ident!("_ev")
    } else {
        format_ident!("ev")
    };
    let closure = scope.loop_item.map_or_else(
        || {
            quote! {
                move |#ev: #ev_ty| {
                    #body
                }
            }
        },
        |_item| {
            quote! {
                move |#ev: #ev_ty| {
                    let __rangular_loop = std::sync::Arc::clone(&__rangular_loop_store.get_value());
                    #body
                }
            }
        },
    );
    hoist.hoist_closure(&closure)
}

struct HoistState {
    expr_lets: Vec<TokenStream>,
    closure_lets: Vec<TokenStream>,
}

impl HoistState {
    const fn new() -> Self {
        Self {
            expr_lets: Vec::new(),
            closure_lets: Vec::new(),
        }
    }

    fn prelude(&self) -> TokenStream {
        let expr_lets = &self.expr_lets;
        let closure_lets = &self.closure_lets;
        quote! {
            #(#expr_lets)*
            #(#closure_lets)*
        }
    }

    fn hoist_expr(&mut self, expr: &Expr) -> TokenStream {
        let n = self.expr_lets.len();
        let id = format_ident!("__rangular_e{n}");
        let init = expr_tokens(expr);
        self.expr_lets.push(quote! {
            let #id: &'static rangular_expr::Expr = Box::leak(Box::new(#init));
        });
        quote! { #id }
    }

    fn hoist_closure(&mut self, closure: &TokenStream) -> TokenStream {
        let n = self.closure_lets.len();
        let id = format_ident!("__rangular_f{n}");
        let closure = closure.clone();
        self.closure_lets.push(quote! {
            let #id = leptos::prelude::StoredValue::new(std::sync::Arc::new({
                let host = host.clone();
                #closure
            }));
        });
        quote! { #id }
    }

    fn hoist_view_closure(&mut self, view: &TokenStream) -> TokenStream {
        let n = self.closure_lets.len();
        let id = format_ident!("__rangular_f{n}");
        self.closure_lets.push(quote! {
            let #id = leptos::prelude::StoredValue::new(std::sync::Arc::new(move || view! { #view }));
        });
        quote! { #id }
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
    let mut hoist = HoistState::new();
    let Some(body) = lower_nodes(&template.nodes, &mut issues, &scope, &templates, &mut hoist)
    else {
        return EmitTokens {
            tokens: proc_macro2::TokenStream::new(),
            issues,
        };
    };
    let prelude = hoist.prelude();
    let ident = format_ident!("{fn_name}");
    let selects = collect_projection_selects(&template.nodes);
    let has_default = has_default_projection(&template.nodes);
    let tokens = if selects.is_empty() && has_projection(&template.nodes) {
        quote! {
            #[allow(clippy::needless_pass_by_value, clippy::redundant_clone)]
            pub fn #ident<H: rangular_host::Host + 'static>(
                host: rangular_aot::HostCell<H>,
                children: Children,
            ) -> impl IntoView {
                #prelude
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
                #prelude
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
                #prelude
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
    hoist: &mut HoistState,
) -> Option<TokenStream> {
    if nodes.is_empty() {
        return Some(quote! {});
    }
    let parts: Vec<_> = nodes
        .iter()
        .filter_map(|node| lower_node(node, issues, scope, templates, hoist))
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
    hoist: &mut HoistState,
) -> Option<TokenStream> {
    match node {
        Node::Element(el) => lower_element(el, issues, scope, templates, hoist),
        Node::Text(text, _) => {
            let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
            Some(quote! { #collapsed })
        }
        Node::Interpolation(expr, _) => {
            let ex = hoist.hoist_expr(expr);
            let args = scope_args(scope);
            let read =
                hoist_host_closure(hoist, scope, &quote! { host.prop_str_scoped(#ex, #args) });
            Some(quote! { {move || #read.get_value()()} })
        }
        Node::Comment(_, _) | Node::NgTemplate(_) => None,
        Node::Projection(Projection { select, .. }) => select.as_ref().map_or_else(
            || Some(quote! { {children()} }),
            |sel| {
                let pname = format_ident!("{}", select_param_name(sel));
                Some(quote! { {#pname()} })
            },
        ),
        Node::If(block) => lower_if(block, issues, scope, templates, hoist),
        Node::For(block) => lower_for(block, issues, templates, hoist),
    }
}

fn lower_element(
    el: &Element,
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
    hoist: &mut HoistState,
) -> Option<TokenStream> {
    if let Some(name) = template_outlet_ref(&el.attrs) {
        let Some(body) = templates.get(name) else {
            issues.push(AotIssue::error(
                "RANG401",
                format!("unknown ngTemplateOutlet ref `{name}`"),
            ));
            return None;
        };
        return lower_nodes(body, issues, scope, templates, hoist);
    }
    if el.tag == "ng-container" {
        return lower_nodes(&el.children, issues, scope, templates, hoist);
    }
    let attrs = lower_attrs(&el.attrs, scope, hoist);
    let children = lower_children(
        &el.children,
        el.self_closing,
        issues,
        scope,
        templates,
        hoist,
    )?;
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
    hoist: &mut HoistState,
) -> Option<TokenStream> {
    if self_closing {
        return Some(quote! {});
    }
    lower_nodes(children, issues, scope, templates, hoist)
}

fn lower_attrs(attrs: &[Attr], scope: &Scope<'_>, hoist: &mut HoistState) -> TokenStream {
    let mut tokens = Vec::new();
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
        tokens.push(lower_one_attr(attr, scope, hoist));
    }
    if tokens.is_empty() {
        quote! {}
    } else {
        quote! { #(#tokens)* }
    }
}

fn lower_one_attr(attr: &Attr, scope: &Scope<'_>, hoist: &mut HoistState) -> TokenStream {
    let args = scope_args(scope);
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
            let ex = hoist.hoist_expr(expr);
            let handler =
                hoist_host_closure(hoist, scope, &quote! { host.eval_bool_scoped(#ex, #args) });
            prop_attr(name, &handler)
        }
        Attr::Property { name, expr, .. } => {
            let ex = hoist.hoist_expr(expr);
            let handler =
                hoist_host_closure(hoist, scope, &quote! { host.prop_str_scoped(#ex, #args) });
            prop_attr(name, &handler)
        }
        Attr::Attribute { name, expr, .. } | Attr::Input { name, expr, .. } => {
            let attr_name = match attr {
                Attr::Input { name, .. } => format!("data-input-{name}"),
                _ => name.clone(),
            };
            let ex = hoist.hoist_expr(expr);
            let handler =
                hoist_host_closure(hoist, scope, &quote! { host.prop_str_scoped(#ex, #args) });
            attr_binding(&attr_name, &handler)
        }
        Attr::Class { name, expr, .. } => {
            let ex = hoist.hoist_expr(expr);
            let handler = hoist_host_closure(
                hoist,
                scope,
                &quote! { host.eval_truthy_scoped(#ex, #args) },
            );
            class_attr(name, &handler)
        }
        Attr::Event { name, expr, .. } => lower_dom_event(name, expr, scope, hoist),
        Attr::Output { name, expr, .. } => {
            let attr_name = format!("data-output-{name}");
            let handler = rangular_parser::event_handler_name(expr);
            static_attr(&attr_name, handler)
        }
        Attr::Ref { .. } => quote! {},
    }
}

fn event_value_tokens(event_name: &str) -> TokenStream {
    if event_name == "error" {
        quote! { String::new() }
    } else {
        quote! {
            {
                use wasm_bindgen::JsCast;
                ev.target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                    .map(|el| el.value())
                    .unwrap_or_default()
            }
        }
    }
}

fn event_param_type(event_name: &str) -> TokenStream {
    match event_name {
        "click" | "dblclick" | "auxclick" => quote! { web_sys::MouseEvent },
        "error" => quote! { web_sys::ErrorEvent },
        _ => quote! { web_sys::Event },
    }
}

fn lower_dom_event(
    name: &str,
    expr: &Expr,
    scope: &Scope<'_>,
    hoist: &mut HoistState,
) -> TokenStream {
    let handler = rangular_parser::event_handler_name(expr);
    let ex = hoist.hoist_expr(expr);
    let args = scope_args(scope);
    let ev_ty = event_param_type(name);
    let event_value = event_value_tokens(name);
    let callback = hoist_event_closure(
        hoist,
        scope,
        name,
        &ev_ty,
        &quote! {
            let event_value = #event_value;
            host.emit_dom_event_call_scoped(#handler, #ex, #name, event_value, #args);
        },
    );
    event_attr(name, &callback)
}

fn lower_if(
    block: &IfBlock,
    issues: &mut Vec<AotIssue>,
    scope: &Scope<'_>,
    templates: &HashMap<String, Vec<Node>>,
    hoist: &mut HoistState,
) -> Option<TokenStream> {
    let cond = hoist.hoist_expr(&block.cond);
    let args = scope_args(scope);
    let then_view = lower_nodes(&block.then_branch, issues, scope, templates, hoist)?;
    if let Some(else_branch) = &block.else_branch {
        let else_view = lower_nodes(else_branch, issues, scope, templates, hoist)?;
        let when = hoist_host_closure(
            hoist,
            scope,
            &quote! { host.eval_truthy_scoped(#cond, #args) },
        );
        let fallback = hoist.hoist_view_closure(&quote! { #else_view });
        Some(quote! {
            <Show
                when=move || #when.get_value()()
                fallback=move || #fallback.get_value()()
            >
                #then_view
            </Show>
        })
    } else {
        let when = hoist_host_closure(
            hoist,
            scope,
            &quote! { host.eval_truthy_scoped(#cond, #args) },
        );
        Some(quote! {
            <Show when=move || #when.get_value()()>
                #then_view
            </Show>
        })
    }
}

fn lower_for(
    block: &ForBlock,
    issues: &mut Vec<AotIssue>,
    templates: &HashMap<String, Vec<Node>>,
    hoist: &mut HoistState,
) -> Option<TokenStream> {
    let iter = hoist.hoist_expr(&block.iter);
    let item_name = block.item.as_str();
    let item_ident = format_ident!("{}", item_name);
    let scope = Scope::with_loop_item(item_name);
    let mut body_hoist = HoistState::new();
    let body = lower_nodes(&block.body, issues, &scope, templates, &mut body_hoist)?;
    let body_prelude = body_hoist.prelude();
    let each = hoist.hoist_closure(&quote! { move || host.eval_list(#iter) });
    let item_lit = item_name;
    let key = block.track.as_ref().map_or_else(
        || quote! { |#item_ident: &String| #item_ident.clone() },
        |track| {
            let tr = hoist.hoist_expr(track);
            let key_fn = hoist.hoist_closure(&quote! {
                move |#item_ident: &String| {
                    let __rangular_item = #item_ident.clone();
                    host.prop_str_scoped(#tr, Some(#item_lit), Some(__rangular_item.as_str()))
                }
            });
            quote! { move |#item_ident: &String| #key_fn.get_value()(#item_ident) }
        },
    );
    Some(quote! {
        <For each=move || #each.get_value()() key=#key let:#item_ident>
            {
                let __rangular_loop_store = leptos::prelude::StoredValue::new(std::sync::Arc::new(
                    #item_ident.clone(),
                ));
                #body_prelude
                view! { #body }
            }
        </For>
    })
}

fn html_name(name: &str) -> TokenStream {
    let mut out = TokenStream::new();
    let mut first = true;
    let mut after_empty = false;
    for part in name.split('-') {
        if part.is_empty() {
            out.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
            out.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
            after_empty = true;
            continue;
        }
        if !first && !after_empty {
            out.extend([TokenTree::Punct(Punct::new('-', Spacing::Joint))]);
        }
        after_empty = false;
        out.extend([TokenTree::Ident(format_ident!("{part}"))]);
        first = false;
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
    let handler = quote! { move || #value.get_value()() };
    prefix_attr("prop", name, &handler)
}

fn attr_binding(name: &str, value: &TokenStream) -> TokenStream {
    let handler = quote! { move || #value.get_value()() };
    let mut out = html_name(name);
    out.extend([TokenTree::Punct(Punct::new('=', Spacing::Joint))]);
    out.extend(handler);
    out
}

fn class_attr(name: &str, value: &TokenStream) -> TokenStream {
    let lit = Literal::string(name);
    let handler = quote! { move || #value.get_value()() };
    quote! { class=(#lit, #handler) }
}

fn event_attr(name: &str, value: &TokenStream) -> TokenStream {
    let handler = quote! { move |ev| #value.get_value()(ev) };
    prefix_attr("on", name, &handler)
}

const fn sanitize_tag(tag: &str) -> &str {
    if tag.is_empty() {
        "div"
    } else {
        tag
    }
}
