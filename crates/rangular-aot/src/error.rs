#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AotIssue {
    pub code: &'static str,
    pub message: String,
}

impl AotIssue {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EmitResult {
    pub code: String,
    pub issues: Vec<AotIssue>,
}

#[derive(Clone, Debug, Default)]
pub struct EmitTokens {
    pub tokens: proc_macro2::TokenStream,
    pub issues: Vec<AotIssue>,
}

impl EmitResult {
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.issues.is_empty()
    }
}

impl EmitTokens {
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.issues.is_empty()
    }
}
