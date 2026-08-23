use rangular_aot::compile;
use syn::parse_file;

fn assert_emits(html: &str, fn_name: &str, label: &str) {
    let out = compile(html, fn_name);
    assert!(out.ok(), "{label}: {:?}", out.issues);
    assert_ne!(out.code, "", "{label}: expected emitted Rust");
    parse_file(&out.code).unwrap_or_else(|err| panic!("{label}: invalid Rust: {err}"));
}

#[test]
fn seed_bar_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/html/seed-bar.html");
    assert_emits(html, "seed_bar_view", "seed-bar");
}

#[test]
fn asset_icon_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/asset-icon/asset-icon.html");
    assert_emits(html, "asset_icon_view", "asset-icon");
}

#[test]
fn color_field_parse_has_nodes() {
    let html = include_str!("../../../tests/fixtures/components/color-field/color-field.html");
    assert!(html.contains("color-field"));
    let parsed = rangular_parser::parse(html, "color-field.html");
    assert!(parsed.ok(), "{:?}", parsed.diagnostics);
    assert!(!parsed.template.nodes.is_empty(), "expected root nodes");
}

#[test]
fn color_field_emits_leptos_view() {
    let html = include_str!("../../../tests/fixtures/components/color-field/color-field.html");
    assert_emits(html, "color_field_view", "color-field");
}

#[test]
fn garbage_input_returns_issues_not_empty_code() {
    let out = compile("<@broken", "broken_view");
    assert!(!out.ok());
    assert_eq!(out.code, "");
}
