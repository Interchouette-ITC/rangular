use crate::span::{line_col, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub file: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn error(
        code: &'static str,
        file: &str,
        source: &str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self::with_severity(Severity::Error, code, file, source, span, message)
    }

    pub fn warning(
        code: &'static str,
        file: &str,
        source: &str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self::with_severity(Severity::Warning, code, file, source, span, message)
    }

    fn with_severity(
        severity: Severity,
        code: &'static str,
        file: &str,
        source: &str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        let (line, col) = line_col(source, span.start);
        let _ = (line, col);
        Self {
            code,
            severity,
            message: format!("{} at {}:{}:{}", message.into(), file, line, col),
            file: file.to_owned(),
            span,
        }
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}
