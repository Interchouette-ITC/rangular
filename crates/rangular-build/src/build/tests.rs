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
    let components = discover(&temp.path().join("components")).unwrap();
    let mut css = String::new();
    let output = compile_components(&components, temp.path(), &mut css).unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(css, "");
}
