use rangular_css::{compile_scss, encapsulate, encapsulate_css, ScopeAttrs};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

fn scope() -> ScopeAttrs {
    ScopeAttrs::new("r0")
}

#[test]
fn compile_scss_drops_bare_host() {
    let out = compile_scss(
        r"
:host { display: contents; }
.seed-bar { display: flex; .btn { margin: 0; } }
",
    );
    assert!(out.ok(), "{:?}", out.issues);
    assert!(!out.css.contains(":host"));
    assert!(!out.css.contains("_ngcontent"));
    assert!(!out.css.contains("_nghost"));
    assert!(out.css.contains(".seed-bar {"));
    assert!(out.css.contains(".seed-bar .btn {"));
}

#[test]
fn compile_scss_host_class_becomes_class() {
    let out = compile_scss(":host.open .panel { color: red; }");
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains(".open .panel"));
    assert!(!out.css.contains(":host"));
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
fn compile_scss_keeps_container_and_has() {
    let out = compile_scss(
        r"
.item-list {
  container-type: inline-size;
}
.item-list:has(.item-list__implicit) {
  outline: 1px solid red;
}
@container (max-width: 20rem) {
  .item-list { flex-direction: column; }
}
",
    );
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("container-type: inline-size"));
    assert!(out.css.contains(".item-list:has(.item-list__implicit)"));
    assert!(out.css.contains("@container (max-width: 20rem)"));
}

#[test]
fn encapsulate_keeps_container_and_has() {
    let out = encapsulate(
        r"
.item-list {
  container-type: inline-size;
}
.item-list:has(.item-list__implicit) {
  outline: 1px solid red;
}
@container (max-width: 20rem) {
  .item-list__row { color: blue; }
}
",
        &scope(),
    );
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("@container (max-width: 20rem)"));
    assert!(out
        .css
        .contains(".item-list[_ngcontent-r0]:has(.item-list__implicit)"));
    assert!(out.css.contains(".item-list__row[_ngcontent-r0]"));
}

#[test]
fn encapsulate_css_scopes_inside_layer() {
    let out = encapsulate_css("@layer components { .x { color: red; } }", &scope());
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("@layer components"));
    assert!(out.css.contains(".x[_ngcontent-r0]"));
}

#[test]
fn seed_bar_bootstrap_coexist_fixture() {
    let scss =
        std::fs::read_to_string(fixture_root().join("components/seed-bar/seed-bar.scss")).unwrap();
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

#[test]
fn compile_scss_and_encapsulate_error_paths() {
    let bad_compile = compile_scss(".x { color: ");
    assert!(!bad_compile.ok());
    assert!(bad_compile.issues.iter().any(|i| i.code == "RANG301"));

    let bad_enc = encapsulate(".x { color: ", &scope());
    assert!(!bad_enc.ok());

    let no_brace = encapsulate_css(".x color: red;", &scope());
    assert!(!no_brace.ok());

    let host_fn = encapsulate_css(":host(.open) .panel { color: red; }", &scope());
    assert!(host_fn.ok(), "{:?}", host_fn.issues);
    assert!(host_fn.css.contains("[_nghost-r0].open"));

    let host_empty_fn = encapsulate_css(":host() .panel { color: red; }", &scope());
    assert!(host_empty_fn.ok(), "{:?}", host_empty_fn.issues);

    let malformed_host = encapsulate_css(":host(.open .panel { color: red; }", &scope());
    assert!(!malformed_host.ok());

    let keyframes = encapsulate_css(
        "@keyframes spin { from { opacity: 0; } to { opacity: 1; } }",
        &scope(),
    );
    assert!(keyframes.ok(), "{:?}", keyframes.issues);
    assert!(keyframes.css.contains("@keyframes"));

    let media = encapsulate_css(
        "@media (min-width: 1px) { .panel:hover { color: blue; } }",
        &scope(),
    );
    assert!(media.ok(), "{:?}", media.issues);
    assert!(media.css.contains(".panel[_ngcontent-r0]:hover"));

    let commented = encapsulate_css("/* note */ .x { color: red; }", &scope());
    assert!(commented.ok(), "{:?}", commented.issues);
    assert!(commented.css.contains(".x[_ngcontent-r0]"));

    let flat_host = compile_scss(":host(.open) .panel { color: red; }");
    assert!(flat_host.ok(), "{:?}", flat_host.issues);
    assert!(!flat_host.css.contains(":host"));
    assert!(flat_host.css.contains(".open"));
}

#[test]
fn compile_scss_nested_at_and_passthrough() {
    let out = compile_scss(
        r"
@supports (display: grid) {
  .grid { display: grid; }
}
@font-face { font-family: X; src: local(X); }
",
    );
    assert!(out.ok(), "{:?}", out.issues);
    assert!(out.css.contains("@supports") || out.css.contains("@font-face"));
}
