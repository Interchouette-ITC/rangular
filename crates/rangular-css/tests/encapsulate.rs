use rangular_css::{encapsulate, encapsulate_css, ScopeAttrs};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

fn scope() -> ScopeAttrs {
    ScopeAttrs::new("r0")
}

#[test]
fn host_alone_rewrites() {
    let out = encapsulate(":host { display: block; }", &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("[_nghost-r0]"));
    assert!(!out.css.contains(":host"));
}

#[test]
fn host_with_class_rewrites() {
    let out = encapsulate(":host.asset-icon-host { display: flex; }", &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("[_nghost-r0].asset-icon-host"));
}

#[test]
fn host_function_rewrites() {
    let out = encapsulate(":host(.open) .panel { color: red; }", &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("[_nghost-r0].open"));
    assert!(out.css.contains(".panel[_ngcontent-r0]"));
}

#[test]
fn component_class_gets_content_attr() {
    let out = encapsulate(".color-field__label { font-size: 0.75rem; }", &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains(".color-field__label[_ngcontent-r0]"));
}

#[test]
fn bootstrap_btn_not_scoped() {
    let scss = r"
.btn { color: red; }
.btn.btn-primary { font-weight: 700; }
.seed-bar .btn { margin: 0; }
";
    let out = encapsulate(scss, &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(
        out.css
            .lines()
            .any(|l| l.trim_start().starts_with(".btn {")),
        "standalone .btn must stay unscoped:\n{}",
        out.css
    );
    assert!(
        out.css
            .lines()
            .any(|l| l.trim_start().starts_with(".btn.btn-primary {")),
        "standalone .btn.btn-primary must stay unscoped:\n{}",
        out.css
    );
    assert!(out.css.contains(".seed-bar .btn[_ngcontent-r0]"));
}

#[test]
fn unbalanced_returns_issue_not_panic() {
    let out = encapsulate_css(":host { display: block;", &scope());
    assert!(!out.ok());
    assert!(out.issues.iter().any(|i| i.code == "RANG301"));
}

#[test]
fn scss_nesting_compiles() {
    let out = encapsulate(
        ".seed-bar { .inner { color: red; } &--wide { width: 100%; } }",
        &scope(),
    );
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains(".seed-bar .inner[_ngcontent-r0]"));
    assert!(out.css.contains(".seed-bar--wide[_ngcontent-r0]"));
}

#[test]
fn color_field_scss_fixture() {
    let scss =
        std::fs::read_to_string(fixture_root().join("components/color-field/color-field.scss"))
            .unwrap();
    let out = encapsulate(&scss, &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("[_nghost-r0]"));
    assert!(out.css.contains(".color-field__label[_ngcontent-r0]"));
    assert!(!out.css.contains(":host"));
}

#[test]
fn asset_icon_scss_fixture() {
    let scss =
        std::fs::read_to_string(fixture_root().join("components/asset-icon/asset-icon.scss"))
            .unwrap();
    let out = encapsulate(&scss, &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("[_nghost-r0].asset-icon-host"));
    assert!(out.css.contains(".asset-icon[_ngcontent-r0]"));
}

#[test]
fn chrome_header_scss_fixture_media() {
    let scss =
        std::fs::read_to_string(fixture_root().join("components/chrome-header/chrome-header.scss"))
            .unwrap();
    let out = encapsulate(&scss, &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("@media"));
    assert!(out.css.contains(".chrome-header[_ngcontent-r0]"));
}

#[test]
fn seed_bar_bootstrap_coexist_fixture() {
    let scss = std::fs::read_to_string(fixture_root().join("scss/seed-bar-coexist.scss")).unwrap();
    let out = encapsulate(&scss, &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("[_nghost-r0]"));
    assert!(out.css.contains(".seed-bar[_ngcontent-r0]"));
    assert!(
        out.css
            .lines()
            .any(|l| l.trim_start().starts_with(".btn {")),
        "{}",
        out.css
    );
    assert!(out.css.contains(".btn-primary") || out.css.contains(".btn.btn-primary"));
    assert!(!out.css.contains(".btn-secondary[_ngcontent"));
}
