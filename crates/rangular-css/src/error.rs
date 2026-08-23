#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssIssue {
    pub code: &'static str,
    pub message: String,
}

impl CssIssue {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CssResult {
    pub css: String,
    pub issues: Vec<CssIssue>,
}

impl CssResult {
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.issues.is_empty()
    }
}
