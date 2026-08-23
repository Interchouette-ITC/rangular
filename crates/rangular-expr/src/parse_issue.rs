#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseIssue {
    pub code: &'static str,
    pub severity: IssueSeverity,
    pub message: &'static str,
    pub start: usize,
    pub end: usize,
}

impl ParseIssue {
    #[must_use]
    pub const fn error(
        code: &'static str,
        message: &'static str,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            code,
            severity: IssueSeverity::Error,
            message,
            start,
            end,
        }
    }

    #[must_use]
    pub const fn warning(
        code: &'static str,
        message: &'static str,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            code,
            severity: IssueSeverity::Warning,
            message,
            start,
            end,
        }
    }
}

pub struct ParseResult {
    pub expr: Option<crate::ast::Expr>,
    pub issues: Vec<ParseIssue>,
}
