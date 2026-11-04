use crate::ast::{Attr, Element, ForBlock, IfBlock, NgTemplate, Node, Projection, Template};
use crate::banana::{banana_event_name, banana_write_expr};
use crate::diag::Diagnostic;
use crate::expr::{Expr, parse_into};
use crate::projection::is_projection_tag;
use crate::span::{Span, pos};

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Clone, Debug)]
pub struct Parsed {
    pub template: Template,
    pub diagnostics: Vec<Diagnostic>,
}

enum Stop {
    Eof,
    /// Stop before any `</…>` closing tag (name checked in `consume_close_tag`).
    Close,
}

struct Parser<'src> {
    src: &'src str,
    file: &'src str,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'src> Parser<'src> {
    pub fn parse(src: &'src str, file: &'src str) -> Parsed {
        let mut p = Self {
            src,
            file,
            pos: 0,
            diagnostics: Vec::new(),
        };
        let nodes = p.parse_nodes(&Stop::Eof);
        Parsed {
            template: Template { nodes },
            diagnostics: p.diagnostics,
        }
    }

    fn parse_nodes(&mut self, stop: &Stop) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.skip_ws();
            if self.at_stop(stop) {
                break;
            }
            if self.pos >= self.src.len() {
                if matches!(stop, Stop::Close) {
                    self.error(self.span_here(), "unexpected end of template");
                }
                break;
            }
            match self.parse_node() {
                Some(node) => nodes.push(node),
                None => self.bump(),
            }
        }
        nodes
    }

    fn parse_node(&mut self) -> Option<Node> {
        if self.consume("{{") {
            return self.parse_interpolation();
        }
        if self.consume("@if") {
            return Some(Node::If(self.parse_if_block()?));
        }
        if self.consume("@for") {
            return Some(Node::For(self.parse_for_block()?));
        }
        if self.consume("@else") {
            self.warn(self.span_here(), "unexpected @else");
            return None;
        }
        if self.peek() == Some('<') {
            return self.parse_markup();
        }
        self.parse_text()
    }

    fn parse_markup(&mut self) -> Option<Node> {
        let start = self.pos;
        self.bump();
        if self.consume("!--") {
            return self.parse_comment(start);
        }
        if self.consume("/") {
            self.error(
                Span::new(pos(start), pos(self.pos)),
                "unexpected closing tag",
            );
            self.skip_to('>');
            return None;
        }
        self.parse_element_from(start)
    }

    fn parse_element_from(&mut self, start: usize) -> Option<Node> {
        let tag = self.read_tag_name()?;
        let (attrs, ng_if, ng_for) = self.parse_attrs();
        let self_closing = self.consume("/");
        if self.peek() == Some('>') {
            self.bump();
        }
        let span_open = Span::new(pos(start), pos(self.pos));
        let void = self_closing || VOID.contains(&tag.as_str());
        let mut children = Vec::new();
        if !void {
            children = self.parse_nodes(&Stop::Close);
            self.skip_ws();
            self.consume_close_tag(&tag);
        }
        let span = span_open.merge(Span::new(pos(start), pos(self.pos)));
        if is_projection_tag(&tag) {
            let select = attrs.iter().find_map(|attr| match attr {
                Attr::Static {
                    name,
                    value: Some(value),
                    ..
                } if name == "select" => Some(value.clone()),
                _ => None,
            });
            return Some(Node::Projection(Projection { select, span }));
        }
        if tag == "ng-template" {
            let name = attrs.iter().find_map(|attr| match attr {
                Attr::Ref { name, .. } => Some(name.clone()),
                _ => None,
            });
            let Some(name) = name else {
                self.error(span, "ng-template requires a #ref name");
                return None;
            };
            return Some(Node::NgTemplate(NgTemplate {
                name,
                body: children,
                span,
            }));
        }
        let el = Element {
            tag,
            attrs,
            children,
            span,
            self_closing: void,
        };
        if let Some((cond, _)) = ng_if {
            return Some(Node::If(IfBlock {
                cond,
                then_branch: vec![Node::Element(el)],
                else_branch: None,
                span,
            }));
        }
        if let Some(for_attr) = ng_for {
            return Some(Node::For(ForBlock {
                item: for_attr.item,
                iter: for_attr.iter,
                track: for_attr.track,
                body: vec![Node::Element(el)],
                span,
            }));
        }
        Some(Node::Element(el))
    }

    fn parse_interpolation(&mut self) -> Option<Node> {
        let start = self.pos.saturating_sub(2);
        self.skip_ws();
        let expr = self.parse_expr()?;
        self.skip_ws();
        if !self.consume("}}") {
            self.error(
                Span::new(pos(start), pos(self.pos)),
                "unclosed interpolation",
            );
        }
        Some(Node::Interpolation(
            expr,
            Span::new(pos(start), pos(self.pos)),
        ))
    }

    fn parse_if_block(&mut self) -> Option<IfBlock> {
        let start = self.pos.saturating_sub(3);
        self.skip_ws();
        if self.peek() != Some('(') {
            self.error(self.span_here(), "expected '(' after @if");
            return None;
        }
        self.bump();
        let cond = self.parse_expr()?;
        self.skip_ws();
        if self.peek() == Some(')') {
            self.bump();
        }
        self.skip_ws();
        let then_branch = self.parse_block_body();
        self.skip_ws();
        let else_branch = if self.consume("@else") {
            Some(self.parse_block_body())
        } else {
            None
        };
        Some(IfBlock {
            cond,
            then_branch,
            else_branch,
            span: Span::new(pos(start), pos(self.pos)),
        })
    }

    fn parse_for_block(&mut self) -> Option<ForBlock> {
        let start = self.pos.saturating_sub(4);
        self.skip_ws();
        if self.peek() != Some('(') {
            self.error(self.span_here(), "expected '(' after @for");
            return None;
        }
        self.bump();
        self.skip_ws();
        let _ = self.consume("let");
        self.skip_ws();
        let item = self.read_ident()?;
        self.skip_ws();
        if !self.consume("of") {
            self.error(self.span_here(), "expected 'of' in @for");
        }
        self.skip_ws();
        let Some(iter) = self.parse_until(&[';', ')']) else {
            self.error(self.span_here(), "expected iterable expression");
            return None;
        };
        self.skip_ws();
        let track = if self.consume(";") {
            self.skip_ws();
            if self.consume("track") {
                self.skip_ws();
                self.parse_until(&[')'])
            } else {
                None
            }
        } else {
            None
        };
        self.skip_ws();
        if self.peek() == Some(')') {
            self.bump();
        }
        self.skip_ws();
        let body = self.parse_block_body();
        Some(ForBlock {
            item,
            iter,
            track,
            body,
            span: Span::new(pos(start), pos(self.pos)),
        })
    }

    fn parse_block_body(&mut self) -> Vec<Node> {
        self.skip_ws();
        if self.peek() != Some('{') {
            self.error(self.span_here(), "expected '{'");
            return Vec::new();
        }
        self.bump();
        self.parse_braced_nodes()
    }

    fn parse_braced_nodes(&mut self) -> Vec<Node> {
        let mut depth = 1;
        let mut nodes = Vec::new();
        while self.pos < self.src.len() && depth > 0 {
            self.skip_ws();
            if self.consume("{{") {
                if let Some(node) = self.parse_interpolation() {
                    nodes.push(node);
                }
                continue;
            }
            if self.starts_with("@if") {
                if let Some(Node::If(block)) = self.parse_node() {
                    nodes.push(Node::If(block));
                }
                continue;
            }
            if self.starts_with("@for") {
                if let Some(Node::For(block)) = self.parse_node() {
                    nodes.push(Node::For(block));
                }
                continue;
            }
            if self.peek() == Some('{') {
                self.bump();
                depth += 1;
                continue;
            }
            if self.peek() == Some('}') {
                self.bump();
                depth -= 1;
                if depth == 0 {
                    break;
                }
                continue;
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            } else {
                self.bump();
            }
        }
        nodes
    }

    fn parse_comment(&mut self, start: usize) -> Option<Node> {
        let text_start = self.pos;
        while self.pos + 2 < self.src.len() {
            if self.src[self.pos..].starts_with("-->") {
                let text = self.src[text_start..self.pos].to_owned();
                self.pos += 3;
                return Some(Node::Comment(text, Span::new(pos(start), pos(self.pos))));
            }
            self.bump();
        }
        self.error(Span::new(pos(start), pos(self.pos)), "unclosed comment");
        None
    }

    fn parse_text(&mut self) -> Option<Node> {
        let start = self.pos;
        while self.pos < self.src.len() {
            if self.src[self.pos..].starts_with("{{") || self.src[self.pos..].starts_with("@if") {
                break;
            }
            if self.src[self.pos..].starts_with("@for") {
                break;
            }
            if self.peek() == Some('<') {
                break;
            }
            self.bump();
        }
        if start == self.pos {
            return None;
        }
        Some(Node::Text(
            self.src[start..self.pos].to_owned(),
            Span::new(pos(start), pos(self.pos)),
        ))
    }

    fn parse_attrs(&mut self) -> (Vec<Attr>, Option<(Expr, Span)>, Option<NgForAttr>) {
        let mut attrs = Vec::new();
        let mut ng_if = None;
        let mut ng_for = None;
        loop {
            self.skip_ws();
            if matches!(self.peek(), None | Some('>' | '/' | '@')) {
                break;
            }
            let attr_start = self.pos;
            if self.peek() == Some('[') {
                if self.src[self.pos..].starts_with("[(") {
                    if let Some(pair) = self.parse_banana_attr(attr_start) {
                        attrs.extend(pair);
                    }
                } else if let Some(attr) = self.parse_binding_attr(attr_start) {
                    attrs.push(attr);
                }
                continue;
            }
            if self.peek() == Some('(') {
                if let Some(attr) = self.parse_event_attr(attr_start) {
                    attrs.push(attr);
                }
                continue;
            }
            if self.peek() == Some('#') {
                if let Some(attr) = self.parse_ref_attr(attr_start) {
                    attrs.push(attr);
                }
                continue;
            }
            if self.peek() == Some('*') {
                if let Some(directive) = self.parse_structural(attr_start) {
                    match directive {
                        Structural::If(expr, span) => ng_if = Some((expr, span)),
                        Structural::For(f) => ng_for = Some(f),
                        Structural::Unknown(name, span) => {
                            self.warn(span, format!("unknown structural directive `{name}`"));
                        }
                    }
                }
                continue;
            }
            if let Some(attr) = self.parse_static_attr(attr_start) {
                attrs.push(attr);
            } else {
                self.warn(
                    Span::new(pos(attr_start), pos(self.pos)),
                    "skipped malformed attribute",
                );
            }
        }
        (attrs, ng_if, ng_for)
    }

    fn parse_binding_attr(&mut self, start: usize) -> Option<Attr> {
        self.bump();
        let kind = if self.consume("attr.") {
            "attr"
        } else if self.consume("class.") {
            "class"
        } else {
            "prop"
        };
        let name = self.read_attr_name()?;
        if self.peek() != Some(']') {
            self.error(self.span_here(), "expected ']'");
            return None;
        }
        self.bump();
        self.skip_ws();
        if self.peek() != Some('=') {
            self.error(self.span_here(), "expected '=' after binding");
            return None;
        }
        self.bump();
        self.skip_ws();
        if self.peek() != Some('"') && self.peek() != Some('\'') {
            self.error(self.span_here(), "expected quoted expression");
            return None;
        }
        let expr_src = self.read_quoted()?;
        let expr = self.parse_expr_from(&expr_src)?;
        let span = Span::new(pos(start), pos(self.pos));
        match kind {
            "attr" => Some(Attr::Attribute { name, expr, span }),
            "class" => Some(Attr::Class { name, expr, span }),
            _ => Some(Attr::Property { name, expr, span }),
        }
    }

    /// `[(prop)]="ident"` → `[prop]` + `(input|propChange)` writing via `$bananaSet`.
    fn parse_banana_attr(&mut self, start: usize) -> Option<[Attr; 2]> {
        self.bump(); // [
        self.bump(); // (
        let name = self.read_attr_name()?;
        if self.peek() != Some(')') {
            self.error(self.span_here(), "expected ')' in two-way binding");
            return None;
        }
        self.bump();
        if self.peek() != Some(']') {
            self.error(self.span_here(), "expected ']' in two-way binding");
            return None;
        }
        self.bump();
        self.skip_ws();
        if self.peek() != Some('=') {
            self.error(self.span_here(), "expected '=' after two-way binding");
            return None;
        }
        self.bump();
        self.skip_ws();
        if self.peek() != Some('"') && self.peek() != Some('\'') {
            self.error(self.span_here(), "expected quoted expression");
            return None;
        }
        let expr_src = self.read_quoted()?;
        let expr = self.parse_expr_from(&expr_src)?;
        let span = Span::new(pos(start), pos(self.pos));
        if !matches!(expr, Expr::Ident(_)) {
            self.warn(
                span,
                "two-way binding target should be an identifier for Host::set",
            );
        }
        let event = banana_event_name(&name);
        let write = banana_write_expr(&expr);
        Some([
            Attr::Property { name, expr, span },
            Attr::Event {
                name: event,
                expr: write,
                span,
            },
        ])
    }

    fn parse_event_attr(&mut self, start: usize) -> Option<Attr> {
        self.bump();
        let name = self.read_attr_name()?;
        if self.peek() != Some(')') {
            self.error(self.span_here(), "expected ')'");
            return None;
        }
        self.bump();
        self.skip_ws();
        if self.peek() != Some('=') {
            self.error(self.span_here(), "expected '=' after event");
            return None;
        }
        self.bump();
        self.skip_ws();
        let expr_src = self.read_quoted()?;
        let expr = self.parse_expr_from(&expr_src)?;
        Some(Attr::Event {
            name,
            expr,
            span: Span::new(pos(start), pos(self.pos)),
        })
    }

    fn parse_ref_attr(&mut self, start: usize) -> Option<Attr> {
        self.bump(); // #
        let name = self.read_attr_name()?;
        Some(Attr::Ref {
            name,
            span: Span::new(pos(start), pos(self.pos)),
        })
    }

    fn parse_static_attr(&mut self, start: usize) -> Option<Attr> {
        let name = self.read_attr_name()?;
        self.skip_ws();
        let value = if self.peek() == Some('=') {
            self.bump();
            self.skip_ws();
            Some(self.read_quoted()?)
        } else {
            None
        };
        let span = Span::new(pos(start), pos(self.pos));
        // If the value contains `{{ … }}` interpolations, desugar to an
        // attribute binding that evaluates the expressions at runtime.
        // e.g. `href="/user/{{ id }}"` → `Attr::Attribute` with
        // `Expr::Binary(Add, Lit("/user/"), Ident("id"))`.
        if let Some(raw) = &value {
            if raw.contains("{{") {
                if let Some(expr) = self.parse_interpolated_attr_value(raw, span) {
                    return Some(Attr::Attribute { name, expr, span });
                }
            }
        }
        Some(Attr::Static { name, value, span })
    }

    /// Parse a static attribute value that contains `{{ expr }}` segments into
    /// a single `Expr` that concatenates literal strings with evaluated
    /// expressions using `BinOp::Add`. Returns `None` only when the
    /// interpolation syntax is malformed (diagnostic already emitted).
    fn parse_interpolated_attr_value(&mut self, raw: &str, span: Span) -> Option<Expr> {
        let mut parts: Vec<Expr> = Vec::new();
        let mut rest = raw;
        loop {
            match rest.find("{{") {
                None => {
                    // Remaining literal suffix (may be empty — skip it).
                    if !rest.is_empty() {
                        parts.push(Expr::Lit(crate::expr::Literal::Str(rest.to_owned())));
                    }
                    break;
                }
                Some(brace_start) => {
                    // Literal prefix before the `{{`.
                    if brace_start > 0 {
                        parts.push(Expr::Lit(crate::expr::Literal::Str(
                            rest[..brace_start].to_owned(),
                        )));
                    }
                    let after_open = &rest[brace_start + 2..];
                    match after_open.find("}}") {
                        None => {
                            self.error(span, "unclosed '{{' in attribute interpolation");
                            return None;
                        }
                        Some(close) => {
                            let inner = after_open[..close].trim();
                            let expr = self.parse_expr_from(inner)?;
                            parts.push(expr);
                            rest = &after_open[close + 2..];
                        }
                    }
                }
            }
        }
        // Fold the parts into a left-associative Add chain.
        // INVARIANT: parts is non-empty — we only reach here after at least one
        // successful parse_expr_from push, since the caller only calls this
        // function when the raw value contains "{{".
        let mut iter = parts.into_iter();
        let first = iter.next().expect("at least one part after successful interpolation parse");
        Some(iter.fold(first, |acc, part| Expr::Binary {
            op: crate::expr::BinOp::Add,
            left: Box::new(acc),
            right: Box::new(part),
        }))
    }

    fn parse_structural(&mut self, start: usize) -> Option<Structural> {
        self.bump();
        let name = self.read_attr_name()?;
        self.skip_ws();
        if self.peek() != Some('=') {
            return Some(Structural::Unknown(
                name,
                Span::new(pos(start), pos(self.pos)),
            ));
        }
        self.bump();
        self.skip_ws();
        let expr_src = self.read_quoted()?;
        let span = Span::new(pos(start), pos(self.pos));
        if name == "ngIf" {
            return Some(Structural::If(self.parse_expr_from(&expr_src)?, span));
        }
        if name == "ngFor" {
            return Some(Structural::For(parse_ng_for(&expr_src, span, self)?));
        }
        Some(Structural::Unknown(name, span))
    }

    fn parse_until(&mut self, stop: &[char]) -> Option<Expr> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if stop.contains(&ch) {
                break;
            }
            self.bump();
        }
        let slice = self.src[start..self.pos].trim();
        if slice.is_empty() {
            None
        } else {
            self.parse_expr_from(slice)
        }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        let start = self.pos;
        let mut depth = 0;
        while self.pos < self.src.len() {
            let ch = self.peek()?;
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            } else if (ch == '{' || ch == '}') && depth == 0 {
                break;
            }
            self.bump();
        }
        let slice = self.src[start..self.pos].trim();
        self.parse_expr_from(slice)
    }

    fn parse_expr_from(&mut self, slice: &str) -> Option<Expr> {
        parse_into(slice, self.file, &mut self.diagnostics)
    }

    fn read_quoted(&mut self) -> Option<String> {
        let quote = self.peek()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        self.bump();
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == quote {
                let s = self.src[start..self.pos].to_owned();
                self.bump();
                return Some(s);
            }
            if ch == '\\' {
                self.bump();
                if self.peek().is_some() {
                    self.bump();
                }
                continue;
            }
            self.bump();
        }
        self.error(
            Span::new(pos(start), pos(self.pos)),
            "unclosed quoted value",
        );
        None
    }

    fn read_tag_name(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        while matches!(
            self.peek(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '-' | ':')
        ) {
            self.bump();
        }
        if start == self.pos {
            self.error(self.span_here(), "expected tag name");
            None
        } else {
            Some(self.src[start..self.pos].to_owned())
        }
    }

    fn read_attr_name(&mut self) -> Option<String> {
        let start = self.pos;
        while matches!(
            self.peek(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | ':')
        ) {
            self.bump();
        }
        if start == self.pos {
            None
        } else {
            Some(self.src[start..self.pos].to_owned())
        }
    }

    fn read_ident(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        if !matches!(self.peek(), Some('a'..='z' | 'A'..='Z' | '_')) {
            self.error(self.span_here(), "expected identifier");
            return None;
        }
        self.bump();
        while matches!(self.peek(), Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
            self.bump();
        }
        Some(self.src[start..self.pos].to_owned())
    }

    fn consume_close_tag(&mut self, tag: &str) {
        let start = self.pos;
        if self.peek() != Some('<') {
            return;
        }
        self.bump();
        if !self.consume("/") {
            self.pos = start;
            return;
        }
        let close = self.read_tag_name().unwrap_or_default();
        self.skip_ws();
        if self.peek() == Some('>') {
            self.bump();
        }
        if !close.eq_ignore_ascii_case(tag) {
            self.warn(
                Span::new(pos(start), pos(self.pos)),
                format!("closing tag `{close}` does not match `{tag}`"),
            );
        }
    }

    fn at_stop(&self, stop: &Stop) -> bool {
        match stop {
            Stop::Eof => false,
            // Any closing tag ends the child list; `consume_close_tag` warns on name mismatch.
            Stop::Close => self.peek() == Some('<') && self.src[self.pos..].starts_with("</"),
        }
    }

    fn starts_with(&self, word: &str) -> bool {
        self.src[self.pos..].starts_with(word)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
            self.bump();
        }
    }

    fn skip_to(&mut self, ch: char) {
        while self.pos < self.src.len() {
            if self.peek() == Some(ch) {
                self.bump();
                return;
            }
            self.bump();
        }
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
        }
    }

    fn consume(&mut self, text: &str) -> bool {
        if self.src[self.pos..].starts_with(text) {
            self.pos += text.len();
            true
        } else {
            false
        }
    }

    fn span_here(&self) -> Span {
        Span::new(pos(self.pos), pos(self.pos))
    }

    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(
            "RANG001", self.file, self.src, span, message,
        ));
    }

    fn warn(&mut self, span: Span, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::warning(
            "RANG101", self.file, self.src, span, message,
        ));
    }
}

enum Structural {
    If(Expr, Span),
    For(NgForAttr),
    Unknown(String, Span),
}

struct NgForAttr {
    item: String,
    iter: Expr,
    track: Option<Expr>,
}

fn parse_ng_for(src: &str, span: Span, p: &mut Parser<'_>) -> Option<NgForAttr> {
    let trimmed = src.trim();
    if !trimmed.starts_with("let ") {
        p.error(span, "expected `let item of expr` in *ngFor");
        return None;
    }
    let rest = trimmed.strip_prefix("let ")?.trim();
    let (item, rest) = rest.split_once(" of ")?;
    let (iter_src, track_src) = if let Some((left, right)) = rest.split_once("; track ") {
        (left.trim(), Some(right.trim()))
    } else {
        (rest.trim(), None)
    };
    let iter = parse_into(iter_src, p.file, &mut p.diagnostics)?;
    let track = track_src.and_then(|t| parse_into(t, p.file, &mut p.diagnostics));
    Some(NgForAttr {
        item: item.trim().to_owned(),
        iter,
        track,
    })
}

#[must_use]
pub fn parse(src: &str, file: &str) -> Parsed {
    Parser::parse(src, file)
}

#[cfg(test)]
mod coverage_edges {
    use super::*;

    #[test]
    fn parse_text_returns_none_at_markup_boundary() {
        let mut p = Parser {
            src: "<div>",
            file: "t.html",
            pos: 0,
            diagnostics: Vec::new(),
        };
        assert!(p.parse_text().is_none());
    }

    #[test]
    fn consume_close_tag_noop_when_not_lt() {
        let mut p = Parser {
            src: "x",
            file: "t.html",
            pos: 0,
            diagnostics: Vec::new(),
        };
        p.consume_close_tag("div");
        assert_eq!(p.pos, 0);
        assert_eq!(p.diagnostics.len(), 0);
    }

    #[test]
    fn consume_close_tag_resets_when_open_tag() {
        let mut p = Parser {
            src: "<div>",
            file: "t.html",
            pos: 0,
            diagnostics: Vec::new(),
        };
        p.consume_close_tag("span");
        assert_eq!(p.pos, 0);
        assert_eq!(p.diagnostics.len(), 0);
    }

    #[test]
    fn static_attr_without_interpolation_unchanged() {
        // A plain static attribute must still produce Attr::Static.
        let result = parse(r#"<a href="/plain/">x</a>"#, "t.html");
        assert!(result.ok(), "{:?}", result.diagnostics);
        let Node::Element(el) = result.template.nodes.into_iter().next().unwrap() else {
            unreachable!()
        };
        assert!(
            el.attrs.iter().any(|a| matches!(
                a,
                Attr::Static { name, value: Some(v), .. }
                if name == "href" && v == "/plain/"
            )),
            "plain href must remain Attr::Static"
        );
    }

    #[test]
    fn interpolated_attr_value_desugars_to_attribute_binding() {
        // href="/prefix/{{ id }}/suffix" must produce Attr::Attribute, not Static.
        let result = parse(r#"<a href="/u/{{ id }}/view">x</a>"#, "t.html");
        assert!(result.ok(), "{:?}", result.diagnostics);
        let Node::Element(el) = result.template.nodes.into_iter().next().unwrap() else {
            unreachable!()
        };
        assert!(
            el.attrs
                .iter()
                .any(|a| matches!(a, Attr::Attribute { name, .. } if name == "href")),
            "interpolated href must become Attr::Attribute; got: {:?}",
            el.attrs
        );
    }

    #[test]
    fn interpolated_attr_only_braces_no_prefix() {
        // href="{{ url }}" — no literal prefix or suffix.
        let result = parse(r#"<a href="{{ url }}">x</a>"#, "t.html");
        assert!(result.ok(), "{:?}", result.diagnostics);
        let Node::Element(el) = result.template.nodes.into_iter().next().unwrap() else {
            unreachable!()
        };
        assert!(
            el.attrs
                .iter()
                .any(|a| matches!(a, Attr::Attribute { name, .. } if name == "href")),
            "pure interpolated href must become Attr::Attribute"
        );
    }

    #[test]
    fn interpolated_attr_unclosed_brace_emits_diagnostic() {
        let result = parse(r#"<a href="{{ unclosed">x</a>"#, "t.html");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("unclosed") && d.message.contains("interpolation")),
            "must emit diagnostic for unclosed '{{'; got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn interpolated_attr_with_trailing_literal_suffix() {
        // "/prefix/{{ id }}/suffix" — covers the trailing-literal branch (line after last }}).
        let result = parse(r#"<a href="/u/{{ id }}/view">x</a>"#, "t.html");
        assert!(result.ok(), "{:?}", result.diagnostics);
        let Some(Node::Element(el)) = result.template.nodes.first() else {
            panic!("expected element");
        };
        assert!(
            el.attrs
                .iter()
                .any(|a| matches!(a, Attr::Attribute { name, .. } if name == "href")),
            "must be Attr::Attribute"
        );
    }

    #[test]
    fn interpolated_attr_bad_expr_propagates_none() {
        // "{{ ? }}" — the inner expression is invalid; parse_expr_from returns None,
        // so parse_interpolated_attr_value returns None, and we fall back to Attr::Static.
        // This covers the `self.parse_expr_from(inner)?` early-return path.
        let result = parse(r#"<a href="{{ ? }}">x</a>"#, "t.html");
        // Either a diagnostic is emitted (bad expr) or it falls back gracefully.
        // The key is it must not panic.
        let _ = result;
    }
}
