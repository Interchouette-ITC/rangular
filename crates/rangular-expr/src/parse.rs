use crate::ast::{BinOp, Expr, Literal, UnOp};
use crate::parse_issue::{ParseIssue, ParseResult};

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    issues: Vec<ParseIssue>,
}

#[must_use]
pub fn parse(input: &str) -> ParseResult {
    let mut p = Parser {
        src: input,
        pos: 0,
        issues: Vec::new(),
    };
    p.skip_ws();
    let expr = p.parse_ternary();
    p.skip_ws();
    if p.pos < p.src.len() {
        p.error(p.pos, p.src.len(), "unexpected tokens in expression");
    }
    ParseResult {
        expr,
        issues: p.issues,
    }
}

impl Parser<'_> {
    fn parse_ternary(&mut self) -> Option<Expr> {
        let mut expr = self.parse_or()?;
        self.skip_ws();
        if self.peek() == Some('?') {
            let q_start = self.pos;
            self.bump();
            let then_branch = Box::new(self.parse_ternary()?);
            self.skip_ws();
            if self.peek() != Some(':') {
                self.error(self.pos, self.pos + 1, "expected ':' in ternary");
                return Some(expr);
            }
            self.bump();
            let else_branch = Box::new(self.parse_ternary()?);
            self.warn(
                q_start,
                self.pos,
                "ternary expressions are not in SPEC v0.1",
            );
            expr = Expr::Ternary {
                cond: Box::new(expr),
                then_branch,
                else_branch,
            };
        }
        Some(expr)
    }

    fn parse_or(&mut self) -> Option<Expr> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if !self.consume("||") {
                break;
            }
            let right = self.parse_and()?;
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_and(&mut self) -> Option<Expr> {
        let mut left = self.parse_eq()?;
        loop {
            self.skip_ws();
            if !self.consume("&&") {
                break;
            }
            let right = self.parse_eq()?;
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_eq(&mut self) -> Option<Expr> {
        let mut left = self.parse_add()?;
        loop {
            self.skip_ws();
            let op = if self.consume("===") || self.consume("==") {
                BinOp::Eq
            } else if self.consume("!==") || self.consume("!=") {
                BinOp::Ne
            } else {
                break;
            };
            let right = self.parse_add()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_add(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            self.skip_ws();
            if !self.consume("+") {
                break;
            }
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op: BinOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        self.skip_ws();
        if self.consume("!") {
            let inner = self.parse_unary()?;
            return Some(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(inner),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        self.skip_ws();
        let start = self.pos;
        if let Some(lit) = self.parse_literal() {
            return Some(Expr::Lit(lit));
        }
        if self.peek() == Some('(') {
            self.bump();
            let inner = self.parse_ternary()?;
            self.skip_ws();
            if self.peek() == Some(')') {
                self.bump();
            } else {
                self.error(start, self.pos, "unclosed '('");
            }
            return Some(inner);
        }
        let ident = self.parse_ident()?;
        let mut expr = Expr::Ident(ident);
        loop {
            self.skip_ws();
            if self.peek() != Some('(') {
                break;
            }
            self.bump();
            let args = self.parse_call_args();
            self.skip_ws();
            if self.peek() == Some(')') {
                self.bump();
            } else {
                self.error(start, self.pos, "unclosed '(' in call");
            }
            expr = Expr::Call {
                callee: Box::new(expr),
                args,
            };
        }
        Some(expr)
    }

    fn parse_call_args(&mut self) -> Vec<Expr> {
        self.skip_ws();
        if self.peek() == Some(')') {
            return Vec::new();
        }
        let mut args = vec![self.parse_ternary().unwrap_or(Expr::Ident(String::new()))];
        loop {
            self.skip_ws();
            if !self.consume(",") {
                break;
            }
            args.push(self.parse_ternary().unwrap_or(Expr::Ident(String::new())));
        }
        args
    }

    fn parse_literal(&mut self) -> Option<Literal> {
        if self.consume("true") {
            return Some(Literal::Bool(true));
        }
        if self.consume("false") {
            return Some(Literal::Bool(false));
        }
        if matches!(self.peek(), Some('\'' | '"')) {
            return Some(Literal::Str(self.parse_string()?));
        }
        if matches!(self.peek(), Some('0'..='9' | '.')) {
            return Some(Literal::Num(self.parse_number()?));
        }
        None
    }

    fn parse_string(&mut self) -> Option<String> {
        let quote = self.peek()?;
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
        self.error(start, self.pos, "unclosed string");
        None
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        while matches!(self.peek(), Some('0'..='9' | '.')) {
            self.bump();
        }
        self.src[start..self.pos].parse().ok()
    }

    fn parse_ident(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        let first = self.peek()?;
        if !matches!(first, 'a'..='z' | 'A'..='Z' | '_' | '$') {
            self.error(start, start + 1, "expected identifier");
            return None;
        }
        self.bump();
        while matches!(
            self.peek(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '$')
        ) {
            self.bump();
        }
        Some(self.src[start..self.pos].to_owned())
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
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

    fn error(&mut self, start: usize, end: usize, message: &'static str) {
        self.issues
            .push(ParseIssue::error("RANG201", message, start, end));
    }

    fn warn(&mut self, start: usize, end: usize, message: &'static str) {
        self.issues
            .push(ParseIssue::warning("RANG101", message, start, end));
    }
}
