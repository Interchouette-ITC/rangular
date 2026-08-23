use rangular_expr::{eval, Expr};
use rangular_host::{Host, Value};
use rangular_parser::{
    parse, Attr, Diagnostic, Element, ForBlock, IfBlock, Node, Severity, Template,
};

use crate::error::{RenderResult, RuntimeIssue};

#[derive(Clone, Debug, PartialEq)]
pub enum VNode {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Self>,
    },
    Text(String),
}

struct Frame {
    name: String,
    value: Value,
}

struct Ctx<'a, H: Host> {
    host: &'a mut H,
    frames: Vec<Frame>,
    issues: Vec<RuntimeIssue>,
}

/// Parse `source` then render. Parse errors become runtime issues; never panics on content.
#[must_use]
pub fn interpret(source: &str, file: &str, host: &mut impl Host) -> RenderResult {
    let parsed = parse(source, file);
    let mut issues: Vec<RuntimeIssue> = parsed
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(diag_issue)
        .collect();
    if !issues.is_empty() {
        return RenderResult {
            nodes: Vec::new(),
            issues,
        };
    }
    let mut out = render(&parsed.template, host);
    issues.append(&mut out.issues);
    RenderResult {
        nodes: out.nodes,
        issues,
    }
}

/// Interpret a parsed template against `host`.
#[must_use]
pub fn render(template: &Template, host: &mut impl Host) -> RenderResult {
    let mut ctx = Ctx {
        host,
        frames: Vec::new(),
        issues: Vec::new(),
    };
    if template.nodes.is_empty() {
        ctx.issues
            .push(RuntimeIssue::error("RANG501", "empty template"));
        return RenderResult {
            nodes: Vec::new(),
            issues: ctx.issues,
        };
    }
    let nodes = render_nodes(&template.nodes, &mut ctx);
    RenderResult {
        nodes,
        issues: ctx.issues,
    }
}

fn diag_issue(d: &Diagnostic) -> RuntimeIssue {
    RuntimeIssue::error(d.code, d.message.clone())
}

fn render_nodes<H: Host>(nodes: &[Node], ctx: &mut Ctx<'_, H>) -> Vec<VNode> {
    nodes
        .iter()
        .flat_map(|node| render_node(node, ctx))
        .collect()
}

fn render_node<H: Host>(node: &Node, ctx: &mut Ctx<'_, H>) -> Vec<VNode> {
    match node {
        Node::Element(el) => vec![render_element(el, ctx)],
        Node::Text(text, _) => vec![VNode::Text(text.clone())],
        Node::Interpolation(expr, _) => {
            vec![VNode::Text(display_value(&eval_expr(expr, ctx)))]
        }
        Node::Comment(_, _) => Vec::new(),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
    }
}

fn render_element<H: Host>(el: &Element, ctx: &mut Ctx<'_, H>) -> VNode {
    let attrs = render_attrs(&el.attrs, ctx);
    let children = if el.self_closing {
        Vec::new()
    } else {
        render_nodes(&el.children, ctx)
    };
    VNode::Element {
        tag: el.tag.clone(),
        attrs,
        children,
    }
}

fn render_attrs<H: Host>(attrs: &[Attr], ctx: &mut Ctx<'_, H>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for attr in attrs {
        match attr {
            Attr::Static {
                name,
                value: Some(value),
                ..
            } => out.push((name.clone(), value.clone())),
            Attr::Static {
                name, value: None, ..
            } => out.push((name.clone(), String::new())),
            Attr::Property { name, expr, .. } if name == "disabled" => {
                let disabled = match eval_expr(expr, ctx) {
                    Value::Bool(b) => b,
                    other => other.is_truthy(),
                };
                out.push((format!("prop:{name}"), bool_str(disabled)));
            }
            Attr::Property { name, expr, .. } => {
                out.push((format!("prop:{name}"), display_value(&eval_expr(expr, ctx))));
            }
            Attr::Attribute { name, expr, .. } => {
                out.push((format!("attr:{name}"), display_value(&eval_expr(expr, ctx))));
            }
            Attr::Class { name, expr, .. } => {
                if eval_expr(expr, ctx).is_truthy() {
                    out.push((format!("class:{name}"), "true".into()));
                }
            }
            Attr::Event { name, expr, .. } => {
                out.push((format!("on:{name}"), event_label(expr)));
            }
        }
    }
    out
}

fn render_if<H: Host>(block: &IfBlock, ctx: &mut Ctx<'_, H>) -> Vec<VNode> {
    if eval_expr(&block.cond, ctx).is_truthy() {
        render_nodes(&block.then_branch, ctx)
    } else if let Some(else_branch) = &block.else_branch {
        render_nodes(else_branch, ctx)
    } else {
        Vec::new()
    }
}

fn render_for<H: Host>(block: &ForBlock, ctx: &mut Ctx<'_, H>) -> Vec<VNode> {
    let items = match eval_expr(&block.iter, ctx) {
        Value::List(items) => items,
        _ => Vec::new(),
    };
    let mut children = Vec::new();
    for item in items {
        ctx.frames.push(Frame {
            name: block.item.clone(),
            value: item,
        });
        children.extend(render_nodes(&block.body, ctx));
        ctx.frames.pop();
    }
    children
}

fn eval_expr<H: Host>(expr: &Expr, ctx: &mut Ctx<'_, H>) -> Value {
    if let Some(v) = resolve_frame(expr, ctx) {
        return v;
    }
    eval(expr, ctx.host).unwrap_or(Value::Unit)
}

fn resolve_frame<H: Host>(expr: &Expr, ctx: &mut Ctx<'_, H>) -> Option<Value> {
    match expr {
        Expr::Ident(name) => ctx
            .frames
            .iter()
            .rev()
            .find(|f| f.name == *name)
            .map(|f| f.value.clone()),
        Expr::Call { callee, args } if !ctx.frames.is_empty() => {
            let Expr::Ident(callee_name) = callee.as_ref() else {
                return None;
            };
            let values: Vec<Value> = args.iter().map(|arg| eval_expr(arg, ctx)).collect();
            ctx.host.call(callee_name, &values).ok()
        }
        _ => None,
    }
}

fn event_label(expr: &Expr) -> String {
    match expr {
        Expr::Call { callee, .. } => match callee.as_ref() {
            Expr::Ident(name) => name.clone(),
            _ => "handler".into(),
        },
        Expr::Ident(name) => name.clone(),
        _ => "handler".into(),
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Str(s) => s.clone(),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::List(items) => items
            .iter()
            .map(display_value)
            .collect::<Vec<_>>()
            .join(","),
        Value::Unit => String::new(),
    }
}

fn bool_str(b: bool) -> String {
    if b {
        "true".into()
    } else {
        "false".into()
    }
}
