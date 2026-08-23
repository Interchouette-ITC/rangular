#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIssue {
    pub code: &'static str,
    pub message: String,
}

impl RuntimeIssue {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderResult {
    pub nodes: Vec<crate::VNode>,
    pub issues: Vec<RuntimeIssue>,
}

impl RenderResult {
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.issues.is_empty()
    }

    #[must_use]
    pub fn snapshot(&self) -> String {
        crate::snapshot::snapshot(&self.nodes)
    }
}
