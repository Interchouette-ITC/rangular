use std::collections::HashMap;

use rangular_expr::{eval_with_pipes, Expr, PipeRegistry};
use rangular_host::{Host, Value};
use rangular_parser::{
    builtin_tag_io, classify_bindings, collect_ng_templates, collect_projection_selects, parse,
    template_outlet_ref, Attr, Diagnostic, Element, ForBlock, IfBlock, Node, Severity, Template,
};

use crate::error::{RenderResult, RuntimeIssue};
use crate::slots::{ProjectionBag, VNode};

struct Frame {
    name: String,
    value: Value,
}

struct Ctx<'a, H: Host> {
    host: &'a mut H,
    pipes: &'a PipeRegistry,
    frames: Vec<Frame>,
    issues: Vec<RuntimeIssue>,
    slots: &'a ProjectionBag,
    templates: HashMap<String, Vec<Node>>,
}

/// Parse `source` then render. Parse errors become runtime issues; never panics on content.
#[must_use]
pub fn interpret(source: &str, file: &str, host: &mut impl Host) -> RenderResult {
    interpret_with_pipes(source, file, host, PipeRegistry::builtins())
}

/// Like [`interpret`] with an explicit [`PipeRegistry`].
#[must_use]
pub fn interpret_with_pipes(
    source: &str,
    file: &str,
    host: &mut impl Host,
    pipes: &PipeRegistry,
) -> RenderResult {
    interpret_with_slots_and_pipes(source, file, host, &ProjectionBag::default(), pipes)
}

/// Like [`interpret`], inserting flat `slot` roots at default `<ng-content>`
/// (and partitioning when the template has `select`).
#[must_use]
pub fn interpret_with_slot(
    source: &str,
    file: &str,
    host: &mut impl Host,
    slot: &[VNode],
) -> RenderResult {
    interpret_with_slot_and_pipes(source, file, host, slot, PipeRegistry::builtins())
}

/// Like [`interpret_with_slot`] with an explicit [`PipeRegistry`].
#[must_use]
pub fn interpret_with_slot_and_pipes(
    source: &str,
    file: &str,
    host: &mut impl Host,
    slot: &[VNode],
    pipes: &PipeRegistry,
) -> RenderResult {
    let mut parsed = parse(source, file);
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
    classify_bindings(&mut parsed.template, &builtin_tag_io());
    let selects = collect_projection_selects(&parsed.template.nodes);
    let bag = ProjectionBag::from_flat(slot, &selects);
    let mut out = render_with_slots_and_pipes(&parsed.template, host, &bag, pipes);
    issues.append(&mut out.issues);
    RenderResult {
        nodes: out.nodes,
        issues,
    }
}

/// Interpret with an explicit named/default projection bag.
#[must_use]
pub fn interpret_with_slots(
    source: &str,
    file: &str,
    host: &mut impl Host,
    slots: &ProjectionBag,
) -> RenderResult {
    interpret_with_slots_and_pipes(source, file, host, slots, PipeRegistry::builtins())
}

/// Like [`interpret_with_slots`] with an explicit [`PipeRegistry`].
#[must_use]
pub fn interpret_with_slots_and_pipes(
    source: &str,
    file: &str,
    host: &mut impl Host,
    slots: &ProjectionBag,
    pipes: &PipeRegistry,
) -> RenderResult {
    let mut parsed = parse(source, file);
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
    classify_bindings(&mut parsed.template, &builtin_tag_io());
    let mut out = render_with_slots_and_pipes(&parsed.template, host, slots, pipes);
    issues.append(&mut out.issues);
    RenderResult {
        nodes: out.nodes,
        issues,
    }
}

/// Interpret a parsed template against `host`.
#[must_use]
pub fn render(template: &Template, host: &mut impl Host) -> RenderResult {
    render_with_slot(template, host, &[])
}

/// Interpret `template`, projecting flat `slot` roots (partitioned if needed).
#[must_use]
pub fn render_with_slot(template: &Template, host: &mut impl Host, slot: &[VNode]) -> RenderResult {
    render_with_slot_and_pipes(template, host, slot, PipeRegistry::builtins())
}

/// Like [`render_with_slot`] with an explicit [`PipeRegistry`].
#[must_use]
pub fn render_with_slot_and_pipes(
    template: &Template,
    host: &mut impl Host,
    slot: &[VNode],
    pipes: &PipeRegistry,
) -> RenderResult {
    let selects = collect_projection_selects(&template.nodes);
    let bag = ProjectionBag::from_flat(slot, &selects);
    render_with_slots_and_pipes(template, host, &bag, pipes)
}

/// Interpret with a prepared [`ProjectionBag`].
#[must_use]
pub fn render_with_slots(
    template: &Template,
    host: &mut impl Host,
    slots: &ProjectionBag,
) -> RenderResult {
    render_with_slots_and_pipes(template, host, slots, PipeRegistry::builtins())
}

/// Like [`render_with_slots`] with an explicit [`PipeRegistry`].
#[must_use]
pub fn render_with_slots_and_pipes(
    template: &Template,
    host: &mut impl Host,
    slots: &ProjectionBag,
    pipes: &PipeRegistry,
) -> RenderResult {
    let templates: HashMap<String, Vec<Node>> =
        collect_ng_templates(&template.nodes).into_iter().collect();
    let mut ctx = Ctx {
        host,
        pipes,
        frames: Vec::new(),
        issues: Vec::new(),
        slots,
        templates,
    };
    if template.nodes.is_empty() {
        ctx.issues
            .push(RuntimeIssue::error("RANG401", "empty template"));
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
        Node::Element(el) => {
            if let Some(name) = template_outlet_ref(&el.attrs) {
                return stamp_template(name, ctx);
            }
            if el.tag == "ng-container" {
                return render_nodes(&el.children, ctx);
            }
            vec![render_element(el, ctx)]
        }
        Node::Text(text, _) => vec![VNode::Text(text.clone())],
        Node::Interpolation(expr, _) => {
            vec![VNode::Text(display_value(&eval_expr(expr, ctx)))]
        }
        Node::Comment(_, _) | Node::NgTemplate(_) => Vec::new(),
        Node::Projection(proj) => ctx.slots.for_select(proj.select.as_deref()).to_vec(),
        Node::If(block) => render_if(block, ctx),
        Node::For(block) => render_for(block, ctx),
    }
}

fn stamp_template<H: Host>(name: &str, ctx: &mut Ctx<'_, H>) -> Vec<VNode> {
    let Some(body) = ctx.templates.get(name).cloned() else {
        return Vec::new();
    };
    render_nodes(&body, ctx)
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
            Attr::Ref { .. } => {}
            Attr::Property { name, .. } if name == "ngTemplateOutlet" => {}
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
                out.push((
                    format!("on:{name}"),
                    rangular_parser::event_handler_name(expr).to_owned(),
                ));
            }
            Attr::Input { name, expr, .. } => {
                out.push((
                    format!("input:{name}"),
                    display_value(&eval_expr(expr, ctx)),
                ));
            }
            Attr::Output { name, expr, .. } => {
                out.push((
                    format!("output:{name}"),
                    rangular_parser::event_handler_name(expr).to_owned(),
                ));
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
    if let Expr::Pipe { expr, name, args } = expr {
        let left = eval_expr(expr, ctx);
        let arg_vals: Vec<Value> = args.iter().map(|arg| eval_expr(arg, ctx)).collect();
        return ctx
            .pipes
            .apply(name, &left, &arg_vals)
            .unwrap_or(Value::Unit);
    }
    if let Some(v) = resolve_frame(expr, ctx) {
        return v;
    }
    eval_with_pipes(expr, ctx.host, ctx.pipes).unwrap_or(Value::Unit)
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
        Value::Event(payload) => match payload {
            rangular_host::EventPayload::Click => "event:click".into(),
            rangular_host::EventPayload::Input { value } => format!("event:input:{value}"),
            rangular_host::EventPayload::Error => "event:error".into(),
            rangular_host::EventPayload::Custom(inner) => {
                format!("event:custom:{}", display_value(inner))
            }
        },
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
