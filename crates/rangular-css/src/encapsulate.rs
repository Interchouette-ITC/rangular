use crate::error::{CssIssue, CssResult};
use crate::globals::is_global_selector;

/// Attribute names used for emulated encapsulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeAttrs {
    pub host: String,
    pub content: String,
}

impl ScopeAttrs {
    #[must_use]
    pub fn new(scope_id: &str) -> Self {
        Self {
            host: format!("_nghost-{scope_id}"),
            content: format!("_ngcontent-{scope_id}"),
        }
    }
}

/// Compile component **SCSS** to flat CSS without emulated `_ngcontent` /
/// `_nghost` attributes. Use this when the DOM does not carry those attrs
/// (typical Leptos CSR). Bare `:host` rules are dropped.
#[must_use]
pub fn compile_scss(scss: &str) -> CssResult {
    match grass::from_string(scss.to_owned(), &grass::Options::default()) {
        Ok(css) => match flatten_css(&css) {
            Ok(out) => CssResult {
                css: out,
                issues: Vec::new(),
            },
            Err(issue) => CssResult {
                css: String::new(),
                issues: vec![issue],
            },
        },
        Err(err) => CssResult {
            css: String::new(),
            issues: vec![CssIssue::error("RANG301", format!("scss: {err}"))],
        },
    }
}

/// Encapsulate component **SCSS**: compile with grass, rewrite `:host`, scope
/// local selectors. Output is flat CSS for the browser.
///
/// Known Bootstrap utility selectors (`.btn`, `.container`, …) are left
/// unscoped so host-app global sheets still match. SCSS errors are `RANG301`.
#[must_use]
pub fn encapsulate(scss: &str, scope: &ScopeAttrs) -> CssResult {
    match grass::from_string(scss.to_owned(), &grass::Options::default()) {
        Ok(css) => encapsulate_css(&css, scope),
        Err(err) => CssResult {
            css: String::new(),
            issues: vec![CssIssue::error("RANG301", format!("scss: {err}"))],
        },
    }
}

/// Encapsulate already-flat CSS (post-compile). Prefer [`encapsulate`] for
/// component sheets authored as SCSS.
#[must_use]
pub fn encapsulate_css(css: &str, scope: &ScopeAttrs) -> CssResult {
    let css = strip_comments(css);
    match process_block(&css, scope) {
        Ok(out) => CssResult {
            css: out,
            issues: Vec::new(),
        },
        Err(issue) => CssResult {
            css: String::new(),
            issues: vec![issue],
        },
    }
}

fn flatten_css(css: &str) -> Result<String, CssIssue> {
    let css = strip_comments(css);
    flatten_block(&css)
}

fn flatten_block(css: &str) -> Result<String, CssIssue> {
    let mut out = String::new();
    let mut rest = css;
    while !rest.trim().is_empty() {
        let (rule, next) = take_rule(rest)?;
        rest = next;
        if rule.trim().is_empty() {
            continue;
        }
        let rewritten = flatten_rule(&rule)?;
        if rewritten.trim().is_empty() {
            continue;
        }
        out.push_str(&rewritten);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    Ok(out)
}

fn flatten_rule(rule: &str) -> Result<String, CssIssue> {
    let rule = rule.trim();
    let open = rule
        .find('{')
        .ok_or_else(|| CssIssue::error("RANG301", "missing rule body"))?;
    let close = rule
        .rfind('}')
        .ok_or_else(|| CssIssue::error("RANG301", "missing rule end"))?;
    let prelude = rule[..open].trim();
    let body = &rule[open + 1..close];

    if prelude.starts_with('@') {
        return flatten_at_rule(prelude, body);
    }

    let mut selectors = Vec::new();
    for part in prelude.split(',') {
        if let Some(sel) = flatten_selector(part.trim())? {
            selectors.push(sel);
        }
    }
    if selectors.is_empty() {
        return Ok(String::new());
    }
    Ok(format!("{} {{{}}}", selectors.join(", "), body.trim()))
}

fn flatten_at_rule(prelude: &str, body: &str) -> Result<String, CssIssue> {
    let name = prelude
        .split_whitespace()
        .next()
        .unwrap_or("@")
        .to_ascii_lowercase();
    if is_nested_at_rule(&name) {
        let inner = flatten_block(body)?;
        if inner.trim().is_empty() {
            return Ok(String::new());
        }
        return Ok(format!("{prelude} {{\n{inner}}}"));
    }
    Ok(format!("{prelude} {{{body}}}"))
}

fn flatten_selector(selector: &str) -> Result<Option<String>, CssIssue> {
    let sel = selector.trim();
    if sel.is_empty() {
        return Err(CssIssue::error("RANG301", "empty selector"));
    }
    if sel.starts_with(":host") {
        return flatten_host(sel);
    }
    Ok(Some(sel.to_owned()))
}

fn flatten_host(sel: &str) -> Result<Option<String>, CssIssue> {
    let rest = &sel[":host".len()..];
    if rest.is_empty() {
        return Ok(None);
    }

    if rest.starts_with('(') {
        let end = rest
            .find(')')
            .ok_or_else(|| CssIssue::error("RANG301", "malformed :host() selector"))?;
        let inner = rest[1..end].trim();
        let after = rest[end + 1..].trim_start();
        let mut out = inner_suffix(inner);
        if out.is_empty() && after.is_empty() {
            return Ok(None);
        }
        if !after.is_empty() {
            if let Some(desc) = flatten_selector(after)? {
                if out.is_empty() {
                    out = desc;
                } else {
                    out.push(' ');
                    out.push_str(&desc);
                }
            } else if out.is_empty() {
                return Ok(None);
            }
        }
        return Ok(Some(out));
    }

    let (host_part, descendant) = split_descendant(rest);
    let mut out = host_part.to_owned();
    if let Some(desc) = descendant {
        if let Some(d) = flatten_selector(desc)? {
            if out.is_empty() {
                out = d;
            } else {
                out.push(' ');
                out.push_str(&d);
            }
        } else if out.is_empty() {
            return Ok(None);
        }
    }
    if out.is_empty() {
        return Ok(None);
    }
    Ok(Some(out))
}

fn process_block(css: &str, scope: &ScopeAttrs) -> Result<String, CssIssue> {
    let mut out = String::new();
    let mut rest = css;
    while !rest.trim().is_empty() {
        let (rule, next) = take_rule(rest)?;
        rest = next;
        if rule.trim().is_empty() {
            continue;
        }
        out.push_str(&rewrite_rule(&rule, scope)?);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    Ok(out)
}

fn take_rule(input: &str) -> Result<(String, &str), CssIssue> {
    let s = input.trim_start();
    if s.is_empty() {
        return Ok((String::new(), ""));
    }
    let open = s.find('{').ok_or_else(|| {
        CssIssue::error(
            "RANG301",
            format!("expected '{{' in css near `{}`", trunc(s)),
        )
    })?;
    let mut depth = 0usize;
    let bytes = s.as_bytes();
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let end = i + 1;
                    return Ok((s[..end].to_owned(), &s[end..]));
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(CssIssue::error("RANG301", "unbalanced braces in css"))
}

fn rewrite_rule(rule: &str, scope: &ScopeAttrs) -> Result<String, CssIssue> {
    let rule = rule.trim();
    let open = rule
        .find('{')
        .ok_or_else(|| CssIssue::error("RANG301", "missing rule body"))?;
    let close = rule
        .rfind('}')
        .ok_or_else(|| CssIssue::error("RANG301", "missing rule end"))?;
    let prelude = rule[..open].trim();
    let body = &rule[open + 1..close];

    if prelude.starts_with('@') {
        return rewrite_at_rule(prelude, body, scope);
    }

    let selectors = prelude
        .split(',')
        .map(|s| rewrite_selector(s.trim(), scope))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("{} {{{}}}", selectors.join(", "), body.trim()))
}

fn is_nested_at_rule(name: &str) -> bool {
    matches!(name, "@media" | "@supports" | "@container" | "@layer")
}

fn rewrite_at_rule(prelude: &str, body: &str, scope: &ScopeAttrs) -> Result<String, CssIssue> {
    let name = prelude
        .split_whitespace()
        .next()
        .unwrap_or("@")
        .to_ascii_lowercase();
    if is_nested_at_rule(&name) {
        let inner = process_block(body, scope)?;
        return Ok(format!("{prelude} {{\n{inner}}}"));
    }
    // @keyframes, @font-face, @import-like blocks: pass through.
    Ok(format!("{prelude} {{{body}}}"))
}

fn rewrite_selector(selector: &str, scope: &ScopeAttrs) -> Result<String, CssIssue> {
    let sel = selector.trim();
    if sel.is_empty() {
        return Err(CssIssue::error("RANG301", "empty selector"));
    }
    if sel.starts_with(":host") {
        return rewrite_host(sel, scope);
    }
    if is_global_selector(sel) {
        return Ok(sel.to_owned());
    }
    Ok(append_attr(sel, &scope.content))
}

fn rewrite_host(sel: &str, scope: &ScopeAttrs) -> Result<String, CssIssue> {
    let rest = &sel[":host".len()..];
    let host = format!("[{}]", scope.host);

    if rest.is_empty() {
        return Ok(host);
    }

    if rest.starts_with('(') {
        let end = rest
            .find(')')
            .ok_or_else(|| CssIssue::error("RANG301", "malformed :host() selector"))?;
        let inner = rest[1..end].trim();
        let after = rest[end + 1..].trim_start();
        let suffix = inner_suffix(inner);
        let mut out = format!("{host}{suffix}");
        if !after.is_empty() {
            out.push(' ');
            out.push_str(&rewrite_selector(after, scope)?);
        }
        return Ok(out);
    }

    // :host.foo.bar …descendant
    let (host_part, descendant) = split_descendant(rest);
    let mut out = format!("{host}{host_part}");
    if let Some(desc) = descendant {
        out.push(' ');
        out.push_str(&rewrite_selector(desc, scope)?);
    }
    Ok(out)
}

fn inner_suffix(inner: &str) -> String {
    if inner.starts_with('.') || inner.starts_with('[') || inner.starts_with('#') {
        inner.to_owned()
    } else if inner.is_empty() {
        String::new()
    } else {
        format!(".{inner}")
    }
}

fn split_descendant(rest: &str) -> (&str, Option<&str>) {
    rest.find(char::is_whitespace).map_or((rest, None), |idx| {
        (&rest[..idx], Some(rest[idx..].trim_start()))
    })
}

fn append_attr(selector: &str, attr: &str) -> String {
    let mut parts: Vec<&str> = selector.split_whitespace().collect();
    if parts.is_empty() {
        return format!("{selector}[{attr}]");
    }
    let last = parts.pop().unwrap_or(selector);
    let rewritten = if let Some((before, pseudo)) = split_trailing_pseudo(last) {
        format!("{before}[{attr}]{pseudo}")
    } else {
        format!("{last}[{attr}]")
    };
    if parts.is_empty() {
        rewritten
    } else {
        format!("{} {rewritten}", parts.join(" "))
    }
}

fn split_trailing_pseudo(compound: &str) -> Option<(&str, &str)> {
    // Keep :hover / ::before on the attribute-scoped compound.
    let bytes = compound.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            // skip [::] or [:]
            return Some((&compound[..i], &compound[i..]));
        }
        i += 1;
    }
    None
}

fn trunc(s: &str) -> String {
    let t = s.chars().take(40).collect::<String>();
    if s.chars().count() > 40 {
        format!("{t}…")
    } else {
        t
    }
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = i.saturating_add(2).min(bytes.len());
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

#[cfg(test)]
mod coverage_arms {
    use super::{append_attr, take_rule};

    #[test]
    fn append_attr_empty_selector_parts() {
        assert_eq!(append_attr("   ", "x"), "   [x]");
        assert_eq!(append_attr("", "x"), "[x]");
    }

    #[test]
    fn take_rule_empty_input() {
        let (rule, rest) = take_rule("").expect("empty");
        assert_eq!(rule, "");
        assert_eq!(rest, "");
        let (rule, rest) = take_rule("   ").expect("ws");
        assert_eq!(rule, "");
        assert_eq!(rest, "");
    }
}
