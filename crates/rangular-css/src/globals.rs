/// Host-app utility classes that must keep matching global stylesheets.
const GLOBAL_EXACT: &[&str] = &[
    "btn",
    "container",
    "container-fluid",
    "row",
    "col",
    "form-control",
    "form-label",
    "form-select",
    "form-check",
    "input-group",
    "nav",
    "navbar",
    "navbar-brand",
    "nav-link",
    "nav-item",
    "modal",
    "modal-dialog",
    "modal-content",
    "modal-header",
    "modal-body",
    "modal-footer",
    "card",
    "card-body",
    "alert",
    "badge",
    "dropdown",
    "dropdown-menu",
    "dropdown-item",
    "list-group",
    "list-group-item",
    "pagination",
    "table",
    "spinner-border",
    "visually-hidden",
    "d-flex",
    "d-none",
    "d-block",
    "w-100",
    "h-100",
    "m-0",
    "p-0",
    "text-muted",
    "text-center",
];

const GLOBAL_PREFIXES: &[&str] = &[
    "btn-",
    "col-",
    "container-",
    "g-",
    "gx-",
    "gy-",
    "m-",
    "mt-",
    "mb-",
    "ms-",
    "me-",
    "mx-",
    "my-",
    "p-",
    "pt-",
    "pb-",
    "ps-",
    "pe-",
    "px-",
    "py-",
    "fs-",
    "fw-",
    "gap-",
];

/// Returns true when `selector` is only Bootstrap-style global utilities.
#[must_use]
pub fn is_global_selector(selector: &str) -> bool {
    let sel = selector.trim();
    if sel.is_empty() || sel.starts_with(':') || sel.starts_with('[') || sel.starts_with('*') {
        return false;
    }
    sel.split_whitespace().all(compound_is_global)
}

fn compound_is_global(compound: &str) -> bool {
    let compound = strip_pseudo(compound);
    if compound.is_empty() {
        return false;
    }
    if !compound.starts_with('.') {
        return false;
    }
    compound
        .split('.')
        .filter(|p| !p.is_empty())
        .all(class_is_global)
}

fn strip_pseudo(compound: &str) -> &str {
    compound.split(':').next().unwrap_or(compound)
}

fn class_is_global(name: &str) -> bool {
    if GLOBAL_EXACT.contains(&name) {
        return true;
    }
    GLOBAL_PREFIXES.iter().any(|p| name.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btn_is_global() {
        assert!(is_global_selector(".btn"));
        assert!(is_global_selector(".btn.btn-primary"));
        assert!(is_global_selector(".btn:hover"));
        assert!(!is_global_selector(".color-field"));
        assert!(!is_global_selector(".btn .color-field"));
    }
}
