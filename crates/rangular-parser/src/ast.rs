use crate::expr::Expr;
use crate::span::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String, Span),
    Interpolation(Expr, Span),
    Comment(String, Span),
    If(IfBlock),
    For(ForBlock),
    /// Angular-shaped content projection (`<ng-content>` / optional `select`).
    Projection(Projection),
    /// Deferred fragment (`<ng-template #ref>…</ng-template>`).
    NgTemplate(NgTemplate),
}

/// Default or named content projection slot (`<ng-content>` / `select`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub select: Option<String>,
    pub span: Span,
}

/// Named template fragment stamped via `[ngTemplateOutlet]`.
#[derive(Clone, Debug, PartialEq)]
pub struct NgTemplate {
    pub name: String,
    pub body: Vec<Node>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub tag: String,
    pub attrs: Vec<Attr>,
    pub children: Vec<Node>,
    pub span: Span,
    pub self_closing: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attr {
    Static {
        name: String,
        value: Option<String>,
        span: Span,
    },
    Property {
        name: String,
        expr: Expr,
        span: Span,
    },
    Attribute {
        name: String,
        expr: Expr,
        span: Span,
    },
    Class {
        name: String,
        expr: Expr,
        span: Span,
    },
    Event {
        name: String,
        expr: Expr,
        span: Span,
    },
    /// Component `@Input` binding on a registered tag (`[label]="…"`).
    Input {
        name: String,
        expr: Expr,
        span: Span,
    },
    /// Component `@Output` binding on a registered tag (`(muteToggle)="…"`).
    Output {
        name: String,
        expr: Expr,
        span: Span,
    },
    /// Template reference variable (`#card` on `ng-template`).
    Ref { name: String, span: Span },
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfBlock {
    pub cond: Expr,
    pub then_branch: Vec<Node>,
    pub else_branch: Option<Vec<Node>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForBlock {
    pub item: String,
    pub iter: Expr,
    pub track: Option<Expr>,
    pub body: Vec<Node>,
    pub span: Span,
}
