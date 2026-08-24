//! Projection select matching and template-outlet helpers.

use crate::ast::{Attr, Element, Node};
use crate::expr::Expr;

/// True when `select` matches a projected root element (fixture subset).
///
/// Supported: tag name (`header`), class (`.header`), attribute (`[data-slot]` /
/// `[data-slot=main]`).
#[must_use]
pub fn matches_select(tag: &str, attrs: &[(String, String)], select: &str) -> bool {
    let select = select.trim();
    if select.is_empty() {
        return false;
    }
    if let Some(class) = select.strip_prefix('.') {
        return attrs
            .iter()
            .any(|(name, value)| name == "class" && value.split_whitespace().any(|c| c == class));
    }
    if let Some(inner) = select.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        if let Some((attr, expected)) = inner.split_once('=') {
            let expected = expected.trim().trim_matches('"').trim_matches('\'');
            return attrs
                .iter()
                .any(|(name, value)| name == attr.trim() && value == expected);
        }
        let attr = inner.trim();
        return attrs.iter().any(|(name, _)| name == attr);
    }
    tag == select
}

/// Property / structural target name for `[ngTemplateOutlet]="ref"`.
#[must_use]
pub fn template_outlet_ref(attrs: &[Attr]) -> Option<&str> {
    attrs.iter().find_map(|attr| match attr {
        Attr::Property {
            name,
            expr: Expr::Ident(id),
            ..
        } if name == "ngTemplateOutlet" => Some(id.as_str()),
        _ => None,
    })
}

/// Collect unique `select` values from `<ng-content select>` in document order.
#[must_use]
pub fn collect_projection_selects(nodes: &[Node]) -> Vec<String> {
    let mut out = Vec::new();
    walk_selects(nodes, &mut out);
    out
}

fn walk_selects(nodes: &[Node], out: &mut Vec<String>) {
    for node in nodes {
        match node {
            Node::Projection(proj) => {
                if let Some(select) = &proj.select {
                    if !out.iter().any(|s| s == select) {
                        out.push(select.clone());
                    }
                }
            }
            Node::Element(el) => walk_selects(&el.children, out),
            Node::NgTemplate(t) => walk_selects(&t.body, out),
            Node::If(block) => {
                walk_selects(&block.then_branch, out);
                if let Some(else_branch) = &block.else_branch {
                    walk_selects(else_branch, out);
                }
            }
            Node::For(block) => walk_selects(&block.body, out),
            Node::Text(_, _) | Node::Interpolation(_, _) | Node::Comment(_, _) => {}
        }
    }
}

/// Whether the tree has a default (unselected) `<ng-content>`.
#[must_use]
pub fn has_default_projection(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Projection(proj) => proj.select.is_none(),
        Node::Element(el) => has_default_projection(&el.children),
        Node::NgTemplate(t) => has_default_projection(&t.body),
        Node::If(block) => {
            has_default_projection(&block.then_branch)
                || block
                    .else_branch
                    .as_ref()
                    .is_some_and(|n| has_default_projection(n))
        }
        Node::For(block) => has_default_projection(&block.body),
        Node::Text(_, _) | Node::Interpolation(_, _) | Node::Comment(_, _) => false,
    })
}

/// Rust parameter name for a `select` string (`.header` → `slot_header`).
#[must_use]
pub fn select_param_name(select: &str) -> String {
    let trimmed = select
        .trim()
        .trim_start_matches('.')
        .trim_start_matches('[')
        .trim_end_matches(']');
    let cleaned: String = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches('_');
    if cleaned.is_empty() {
        "slot_named".into()
    } else {
        format!("slot_{cleaned}")
    }
}

/// Collect `#ref` ng-template bodies keyed by ref name.
#[must_use]
pub fn collect_ng_templates(nodes: &[Node]) -> Vec<(String, Vec<Node>)> {
    let mut out = Vec::new();
    walk_ng_templates(nodes, &mut out);
    out
}

fn walk_ng_templates(nodes: &[Node], out: &mut Vec<(String, Vec<Node>)>) {
    for node in nodes {
        match node {
            Node::NgTemplate(t) => {
                out.push((t.name.clone(), t.body.clone()));
                walk_ng_templates(&t.body, out);
            }
            Node::Element(el) => walk_ng_templates(&el.children, out),
            Node::If(block) => {
                walk_ng_templates(&block.then_branch, out);
                if let Some(else_branch) = &block.else_branch {
                    walk_ng_templates(else_branch, out);
                }
            }
            Node::For(block) => walk_ng_templates(&block.body, out),
            Node::Projection(_)
            | Node::Text(_, _)
            | Node::Interpolation(_, _)
            | Node::Comment(_, _) => {}
        }
    }
}

/// True when `el` is an `ng-container` used only as an outlet host.
#[must_use]
pub fn is_outlet_container(el: &Element) -> bool {
    el.tag == "ng-container" && template_outlet_ref(&el.attrs).is_some()
}
