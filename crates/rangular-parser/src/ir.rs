//! Structural binding IR shared by AOT and runtime parity tests.
//!
//! Host-independent: tags, binding kinds, handler names, `@if` / `@for` shape.
//! Evaluated DOM snapshots stay in `rangular-runtime`.

use crate::ast::{Attr, Element, ForBlock, IfBlock, Node, Template};
use crate::expr::Expr;

/// One node in the structural binding tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrNode {
    Element {
        tag: String,
        bindings: Vec<IrBinding>,
        children: Vec<Self>,
    },
    Text,
    Interpolation,
    If {
        has_else: bool,
        then_branch: Vec<Self>,
        else_branch: Vec<Self>,
    },
    For {
        item: String,
        has_track: bool,
        body: Vec<Self>,
    },
    Projection {
        has_select: bool,
    },
}

/// Attribute / event binding kind on an element (no evaluated values).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrBinding {
    Static { name: String },
    Property { name: String },
    Attribute { name: String },
    Class { name: String },
    Event { event: String, handler: String },
}

/// Lower a parsed template to structural IR (comments omitted).
#[must_use]
pub fn from_template(template: &Template) -> Vec<IrNode> {
    from_nodes(&template.nodes)
}

fn from_nodes(nodes: &[Node]) -> Vec<IrNode> {
    nodes.iter().filter_map(from_node).collect()
}

fn from_node(node: &Node) -> Option<IrNode> {
    match node {
        Node::Element(el) => Some(from_element(el)),
        Node::Text(_, _) => Some(IrNode::Text),
        Node::Interpolation(_, _) => Some(IrNode::Interpolation),
        Node::Comment(_, _) => None,
        Node::If(block) => Some(from_if(block)),
        Node::For(block) => Some(from_for(block)),
        Node::Projection(proj) => Some(IrNode::Projection {
            has_select: proj.select.is_some(),
        }),
    }
}

fn from_element(el: &Element) -> IrNode {
    let children = if el.self_closing {
        Vec::new()
    } else {
        from_nodes(&el.children)
    };
    IrNode::Element {
        tag: el.tag.clone(),
        bindings: el.attrs.iter().map(from_attr).collect(),
        children,
    }
}

fn from_attr(attr: &Attr) -> IrBinding {
    match attr {
        Attr::Static { name, .. } => IrBinding::Static { name: name.clone() },
        Attr::Property { name, .. } => IrBinding::Property { name: name.clone() },
        Attr::Attribute { name, .. } => IrBinding::Attribute { name: name.clone() },
        Attr::Class { name, .. } => IrBinding::Class { name: name.clone() },
        Attr::Event { name, expr, .. } => IrBinding::Event {
            event: name.clone(),
            handler: event_handler_name(expr).to_owned(),
        },
    }
}

fn from_if(block: &IfBlock) -> IrNode {
    let else_branch = block
        .else_branch
        .as_ref()
        .map_or_else(Vec::new, |nodes| from_nodes(nodes));
    IrNode::If {
        has_else: block.else_branch.is_some(),
        then_branch: from_nodes(&block.then_branch),
        else_branch,
    }
}

fn from_for(block: &ForBlock) -> IrNode {
    IrNode::For {
        item: block.item.clone(),
        has_track: block.track.is_some(),
        body: from_nodes(&block.body),
    }
}

/// Handler name for `(event)="name(...)"` / `(event)="name"`.
///
/// Empty string when the callee is not a plain identifier (same contract as AOT).
#[must_use]
pub fn event_handler_name(expr: &Expr) -> &str {
    match expr {
        Expr::Call { callee, .. } => match callee.as_ref() {
            Expr::Ident(name) => name.as_str(),
            _ => "",
        },
        Expr::Ident(name) => name.as_str(),
        _ => "",
    }
}

/// Stable text form of structural IR for golden / equality tests.
#[must_use]
pub fn snapshot(nodes: &[IrNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        write_node(node, 0, &mut out);
    }
    out
}

fn write_node(node: &IrNode, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    match node {
        IrNode::Text => {
            out.push_str(&pad);
            out.push_str("text\n");
        }
        IrNode::Interpolation => {
            out.push_str(&pad);
            out.push_str("interpolation\n");
        }
        IrNode::Element {
            tag,
            bindings,
            children,
        } => {
            out.push_str(&pad);
            out.push('<');
            out.push_str(tag);
            for binding in bindings {
                out.push(' ');
                write_binding(binding, out);
            }
            if children.is_empty() {
                out.push_str("/>\n");
            } else {
                out.push_str(">\n");
                for child in children {
                    write_node(child, depth + 1, out);
                }
                out.push_str(&pad);
                out.push_str("</");
                out.push_str(tag);
                out.push_str(">\n");
            }
        }
        IrNode::If {
            has_else,
            then_branch,
            else_branch,
        } => {
            out.push_str(&pad);
            out.push_str("@if");
            if *has_else {
                out.push_str(" else");
            }
            out.push('\n');
            for child in then_branch {
                write_node(child, depth + 1, out);
            }
            if *has_else {
                out.push_str(&pad);
                out.push_str("@else\n");
                for child in else_branch {
                    write_node(child, depth + 1, out);
                }
            }
        }
        IrNode::For {
            item,
            has_track,
            body,
        } => {
            out.push_str(&pad);
            out.push_str("@for ");
            out.push_str(item);
            if *has_track {
                out.push_str(" track");
            }
            out.push('\n');
            for child in body {
                write_node(child, depth + 1, out);
            }
        }
        IrNode::Projection { has_select } => {
            out.push_str(&pad);
            out.push_str("ng-content");
            if *has_select {
                out.push_str(" select");
            }
            out.push('\n');
        }
    }
}

fn write_binding(binding: &IrBinding, out: &mut String) {
    match binding {
        IrBinding::Static { name } => {
            out.push_str("static:");
            out.push_str(name);
        }
        IrBinding::Property { name } => {
            out.push_str("prop:");
            out.push_str(name);
        }
        IrBinding::Attribute { name } => {
            out.push_str("attr:");
            out.push_str(name);
        }
        IrBinding::Class { name } => {
            out.push_str("class:");
            out.push_str(name);
        }
        IrBinding::Event { event, handler } => {
            out.push_str("on:");
            out.push_str(event);
            out.push('=');
            out.push('"');
            out.push_str(handler);
            out.push('"');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn seed_bar_ir_lists_handlers() {
        let src = include_str!("../../../tests/fixtures/html/seed-bar.html");
        let parsed = parse(src, "seed-bar.html");
        assert!(parsed.ok(), "{:?}", parsed.diagnostics);
        let ir = from_template(&parsed.template);
        let snap = snapshot(&ir);
        assert!(snap.contains(r#"on:input="seedChange""#), "{snap}");
        assert!(snap.contains(r#"on:click="onGenerate""#), "{snap}");
        assert!(snap.contains(r#"on:click="onRandom""#), "{snap}");
        assert!(snap.contains("prop:value"), "{snap}");
        assert!(snap.contains("prop:disabled"), "{snap}");
    }
}
