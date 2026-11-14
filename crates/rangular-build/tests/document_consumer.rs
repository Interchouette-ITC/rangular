//! Builds and tests a real consumer crate of `rangular-build` document views.

use std::{path::Path, process::Command};

#[test]
fn generated_document_views_compile_and_render_in_a_consumer_crate() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/document-consumer/Cargo.toml");
    let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("document-consumer");
    for args in [
        &["test", "--locked"][..],
        &[
            "clippy",
            "--locked",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ][..],
    ] {
        run_cargo(args, &fixture, &target);
    }
}

fn run_cargo(args: &[&str], fixture: &Path, target: &Path) {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .arg(args[0])
        .arg("--manifest-path")
        .arg(fixture)
        .args(&args[1..])
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("run cargo for the document-consumer fixture");
    assert!(
        output.status.success(),
        "consumer fixture failed: cargo {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
