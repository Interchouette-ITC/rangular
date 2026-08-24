//! Two-way banana `[(prop)]` desugar helpers.

use rangular_expr::Expr;

/// Internal call name for desugared banana writeback (`Host::set`).
pub const BANANA_SET_CALLEE: &str = "$bananaSet";

/// DOM / Angular-like event for a two-way property.
#[must_use]
pub fn banana_event_name(prop: &str) -> String {
    if prop == "value" {
        "input".into()
    } else {
        format!("{prop}Change")
    }
}

/// Build `$bananaSet(target, $event)` for Host writeback.
#[must_use]
pub fn banana_write_expr(target: &Expr) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::Ident(BANANA_SET_CALLEE.to_owned())),
        args: vec![target.clone(), Expr::Ident("$event".into())],
    }
}

/// Binding path written by a desugared banana event handler, if any.
#[must_use]
pub fn banana_set_target(expr: &Expr) -> Option<&str> {
    let Expr::Call { callee, args } = expr else {
        return None;
    };
    let Expr::Ident(name) = callee.as_ref() else {
        return None;
    };
    if name != BANANA_SET_CALLEE {
        return None;
    }
    match args.first() {
        Some(Expr::Ident(path)) => Some(path.as_str()),
        _ => None,
    }
}
