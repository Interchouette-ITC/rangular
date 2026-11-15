use super::{build, compile_components, run, run_at};
use crate::build::{Config, discover};
use crate::test_support::TempDir;
use std::fs;

#[test]
fn builds_component_rust_and_combined_global_and_component_css() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    let components = manifest.path().join("src/components/card");
    fs::create_dir_all(&components).unwrap();
    fs::write(components.join("card.html"), "<p>Hello</p>").unwrap();
    fs::write(components.join("card.scss"), ".card { color: blue; }").unwrap();
    fs::create_dir_all(manifest.path().join("style")).unwrap();
    fs::write(
        manifest.path().join("style/global.scss"),
        "body { color: black; }",
    )
    .unwrap();

    run_at(
        &Config::new("src/components", "style/generated/site.css").global_scss("style/global.scss"),
        manifest.path(),
        out.path(),
    )
    .unwrap();

    let generated = fs::read_to_string(out.path().join("rangular/card_view.rs")).unwrap();
    assert_ne!(generated, "");
    let css = fs::read_to_string(manifest.path().join("style/generated/site.css")).unwrap();
    assert!(css.contains("body"));
    assert!(css.contains("color: black"));
    assert!(css.contains(".card"));
    assert!(css.contains("color: blue"));
}

#[test]
fn reports_component_global_style_and_output_failures() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    let absent_components = Config::new("missing", "site.css");
    assert!(
        run_at(&absent_components, manifest.path(), out.path())
            .unwrap_err()
            .contains("read ")
    );

    fs::create_dir_all(manifest.path().join("components/card")).unwrap();
    fs::write(
        manifest.path().join("components/card/card.html"),
        "{{ broken",
    )
    .unwrap();
    let error = run_at(
        &Config::new("components", "site.css"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("card.html"));

    fs::write(
        manifest.path().join("components/card/card.html"),
        "<p>Valid</p>",
    )
    .unwrap();
    fs::write(manifest.path().join("global.scss"), "body { color: red;").unwrap();
    let error = run_at(
        &Config::new("components", "site.css").global_scss("global.scss"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("global.scss"));

    fs::write(manifest.path().join("global.scss"), "body { color: red; }").unwrap();
    fs::write(
        manifest.path().join("components/card/card.scss"),
        ".card { color: red;",
    )
    .unwrap();
    let error = run_at(
        &Config::new("components", "site.css").global_scss("global.scss"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("components/card/card.scss"));

    fs::write(
        manifest.path().join("components/card/card.scss"),
        ".card { color: red; }",
    )
    .unwrap();
    fs::write(manifest.path().join("blocked"), "file").unwrap();
    let error = run_at(
        &Config::new("components", "blocked/site.css"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("create "));

    let not_a_dir = manifest.path().join("file");
    fs::write(&not_a_dir, "file").unwrap();
    let error = run_at(
        &Config::new("components", "file/site.css"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("create "));

    let error = run_at(
        &Config::new("components", "components/card"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("write "));

    let error = run_at(&Config::new("components", "/"), manifest.path(), out.path()).unwrap_err();
    assert!(error.contains("write /:"));
}

#[test]
fn reports_rust_output_write_errors() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    fs::create_dir_all(manifest.path().join("components/card")).unwrap();
    fs::write(
        manifest.path().join("components/card/card.html"),
        "<p>Valid</p>",
    )
    .unwrap();
    fs::create_dir_all(out.path().join("rangular")).unwrap();
    fs::create_dir_all(out.path().join("rangular/card_view.rs")).unwrap();
    let error = run_at(
        &Config::new("components", "site.css"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("write "));
}

#[test]
fn reports_generated_directory_creation_errors() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    fs::create_dir_all(manifest.path().join("components/card")).unwrap();
    fs::write(
        manifest.path().join("components/card/card.html"),
        "<p>Valid</p>",
    )
    .unwrap();
    fs::write(out.path().join("rangular"), "file blocks directory").unwrap();
    let error = run_at(
        &Config::new("components", "site.css"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("create "));
}

#[test]
#[allow(unsafe_code)]
fn public_build_and_environment_resolution_cover_success_and_failures() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    fs::create_dir_all(manifest.path().join("components/card")).unwrap();
    fs::write(
        manifest.path().join("components/card/card.html"),
        "<p>Valid</p>",
    )
    .unwrap();

    // This is the only test that mutates process environment variables.
    // SAFETY: this test serializes its own environment changes; other crate tests do not access them.
    unsafe {
        std::env::set_var("CARGO_MANIFEST_DIR", manifest.path());
        std::env::set_var("OUT_DIR", out.path());
    }
    build(Config::new("components", "site.css"));

    // Exercise both unset-variable errors from run(), then the public panic wrapper.
    // SAFETY: same single-test environment mutation constraint as above.
    unsafe { std::env::remove_var("OUT_DIR") };
    assert!(matches!(
        run(&Config::new("components", "site.css")),
        Err(error) if error.contains("OUT_DIR is not set")
    ));
    // SAFETY: same single-test environment mutation constraint as above.
    unsafe { std::env::remove_var("CARGO_MANIFEST_DIR") };
    assert!(matches!(
        run(&Config::new("components", "site.css")),
        Err(error) if error.contains("CARGO_MANIFEST_DIR is not set")
    ));

    let panic = std::panic::catch_unwind(|| build(Config::new("components", "site.css")));
    assert!(panic.is_err());
    // SAFETY: restore the process environment to its original state for the test harness.
    unsafe {
        std::env::remove_var("CARGO_MANIFEST_DIR");
        std::env::remove_var("OUT_DIR");
    }
}

#[test]
fn handles_components_without_scss_in_the_compiler_helper() {
    let temp = TempDir::new();
    fs::create_dir_all(temp.path().join("components/component")).unwrap();
    fs::write(
        temp.path().join("components/component/component.html"),
        "<p>Ready</p>",
    )
    .unwrap();
    let components = discover(&temp.path().join("components"), false).unwrap();
    let mut css = String::new();
    let output = compile_components(&components, temp.path(), &mut css).unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(css, "");
}

fn write(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

const DOCUMENT_HTML: &str =
    include_str!("../../tests/fixtures/document-consumer/src/document/document.html");
const HEAD_HTML: &str =
    include_str!("../../tests/fixtures/document-consumer/src/document/head.html");
const BODY_HTML: &str =
    include_str!("../../tests/fixtures/document-consumer/src/document/body.html");

fn document_config() -> Config {
    Config::new("src/components", "style/site.css")
        .global_scss("style/global.scss")
        .document("src/document/document.html")
        .head("src/document/head.html")
        .body("src/document/body.html")
}

fn write_document_package(root: &std::path::Path) {
    write(root, "src/components/card/card.html", "<p>{{ label }}</p>");
    write(
        root,
        "src/components/card/card.scss",
        ".card { color: blue; }",
    );
    write(root, "style/global.scss", "html { color: black; }");
    write(root, "src/document/document.html", DOCUMENT_HTML);
    write(root, "src/document/document.scss", "body { margin: 0; }");
    write(root, "src/document/head.html", HEAD_HTML);
    write(
        root,
        "src/document/head.scss",
        ".ignored-head { color: red; }",
    );
    write(root, "src/document/body.html", BODY_HTML);
    write(root, "src/document/body.scss", "main { padding: 1px; }");
}

#[test]
fn builds_document_head_and_body_views_with_components() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    write_document_package(manifest.path());

    run_at(&document_config(), manifest.path(), out.path()).unwrap();

    let read = |name: &str| fs::read_to_string(out.path().join("rangular").join(name)).unwrap();
    let document = read("document_view.rs");
    assert!(document.contains("pub fn document_view"), "{document}");
    assert!(
        document
            .split_whitespace()
            .collect::<String>()
            .contains("<!DOCTYPEhtml>"),
        "{document}"
    );
    assert!(document.contains("slot_head"), "{document}");
    let head = read("head_view.rs");
    assert!(head.contains("pub fn head_view"), "{head}");
    for tag in ["<title>", "<meta", "<link"] {
        assert!(head.contains(tag), "{tag}: {head}");
    }
    let body = read("body_view.rs");
    assert!(body.contains("pub fn body_view"), "{body}");
    assert!(read("card_view.rs").contains("pub fn card_view"));

    let css = fs::read_to_string(manifest.path().join("style/site.css")).unwrap();
    let position = |needle: &str| {
        css.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing from {css}"))
    };
    assert!(position("color: black") < position("margin: 0"));
    assert!(position("margin: 0") < position("padding: 1px"));
    assert!(position("padding: 1px") < position(".card"));
    assert!(!css.contains("ignored-head"), "{css}");
}

#[test]
fn component_output_is_unchanged_when_document_templates_are_added() {
    let manifest = TempDir::new();
    write_document_package(manifest.path());

    let plain_out = TempDir::new();
    run_at(
        &Config::new("src/components", "style/plain.css").global_scss("style/global.scss"),
        manifest.path(),
        plain_out.path(),
    )
    .unwrap();
    let document_out = TempDir::new();
    run_at(&document_config(), manifest.path(), document_out.path()).unwrap();

    assert_eq!(
        fs::read_to_string(plain_out.path().join("rangular/card_view.rs")).unwrap(),
        fs::read_to_string(document_out.path().join("rangular/card_view.rs")).unwrap()
    );
    assert!(!plain_out.path().join("rangular/head_view.rs").exists());
}

#[test]
fn document_templates_build_without_components_but_components_alone_still_required() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    fs::create_dir_all(manifest.path().join("src/components")).unwrap();
    write(manifest.path(), "src/app/layout.html", BODY_HTML);

    run_at(
        &Config::new("src/components", "site.css").body("src/app/layout.html"),
        manifest.path(),
        out.path(),
    )
    .unwrap();
    assert!(out.path().join("rangular/layout_view.rs").is_file());

    let error = run_at(
        &Config::new("src/components", "site.css"),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("no components found"), "{error}");
}

#[test]
fn head_adapter_routes_head_elements_to_supplied_components() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    write_document_package(manifest.path());

    run_at(
        &document_config().head_adapter(crate::HeadComponentAdapter::new(
            "app_head::Title",
            "app_head::Meta",
            "app_head::Link",
        )),
        manifest.path(),
        out.path(),
    )
    .unwrap();
    let head = fs::read_to_string(out.path().join("rangular/head_view.rs")).unwrap();
    for component in ["app_head::Title", "app_head::Meta", "app_head::Link"] {
        assert!(head.contains(component), "{component}: {head}");
    }

    let error = run_at(
        &document_config().head_adapter(crate::HeadComponentAdapter::new("", "m::M", "l::L")),
        manifest.path(),
        out.path(),
    )
    .unwrap_err();
    assert!(error.contains("RANG404"), "{error}");
}

#[test]
fn reports_invalid_document_template_configurations() {
    let manifest = TempDir::new();
    let out = TempDir::new();
    write_document_package(manifest.path());
    let run = |config: Config| run_at(&config, manifest.path(), out.path()).unwrap_err();

    write(
        manifest.path(),
        "src/document/invalid_head.html",
        "<div>Body</div>",
    );
    let error = run(document_config().head("src/document/invalid_head.html"));
    assert!(error.contains("src/document/invalid_head.html"), "{error}");
    assert!(error.contains("RANG403"), "{error}");

    let error = run(document_config().body("src/document/missing.html"));
    assert!(error.contains("read body template"), "{error}");

    let error = run(document_config().document("src/document/my-document.html"));
    assert!(error.contains("Rust identifier"), "{error}");

    let error = run(document_config().head("src/document/head.txt"));
    assert!(error.contains("must be an `.html` file"), "{error}");

    write(
        manifest.path(),
        "src/components/head/head.html",
        "<p>Head</p>",
    );
    let error = run(document_config());
    assert!(
        error.contains("head template src/document/head.html generates `head_view`")
            && error.contains("collides with component `head`"),
        "{error}"
    );
    fs::remove_dir_all(manifest.path().join("src/components/head")).unwrap();

    write(manifest.path(), "src/other/document.html", BODY_HTML);
    let error = run(document_config().body("src/other/document.html"));
    assert!(
        error.contains("body template src/other/document.html generates `document_view`")
            && error.contains("collides with document template src/document/document.html"),
        "{error}"
    );

    let error = run(Config::new("src/components", "site.css")
        .head_adapter(crate::HeadComponentAdapter::new("t::T", "m::M", "l::L")));
    assert!(
        error.contains("head_adapter requires a head template"),
        "{error}"
    );

    fs::write(
        manifest.path().join("src/document/body.scss"),
        "main { padding: 1px;",
    )
    .unwrap();
    let error = run(document_config());
    assert!(error.contains("src/document/body.scss"), "{error}");
}
