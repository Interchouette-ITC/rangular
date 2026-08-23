pub use rangular_expr::{BinOp, Expr, Literal, UnOp};

use crate::diag::Diagnostic;

pub fn parse_into(input: &str, file: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<Expr> {
    let result = rangular_expr::parse(input);
    for issue in result.issues {
        let span =
            crate::span::Span::new(crate::span::pos(issue.start), crate::span::pos(issue.end));
        match issue.severity {
            rangular_expr::IssueSeverity::Error => {
                diagnostics.push(Diagnostic::error(
                    issue.code,
                    file,
                    input,
                    span,
                    issue.message,
                ));
            }
            rangular_expr::IssueSeverity::Warning => {
                diagnostics.push(Diagnostic::warning(
                    issue.code,
                    file,
                    input,
                    span,
                    issue.message,
                ));
            }
        }
    }
    result.expr
}
