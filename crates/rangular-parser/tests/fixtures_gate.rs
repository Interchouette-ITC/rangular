//! Fixture corpus gate: every planned construct must have a fixture path on disk.

use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

/// New grammar lands only when these paths exist (plan / SPEC growth contract).
const REQUIRED_FIXTURES: &[&str] = &[
    "html/seed-bar.html",
    "html/event-payload.html",
    "html/io-parent.html",
    "components/item-list/item-list.html",
    "components/chrome-header/chrome-header.html",
    "components/color-field/color-field.html",
    "components/asset-icon/asset-icon.html",
    "components/layout-shell/layout-shell.html",
    "components/layout-shell/layout-shell.scss",
    "components/io-child/io-child.html",
    "components/io-child/io-child.scss",
];

#[test]
fn required_fixtures_exist() {
    let root = fixture_root();
    for rel in REQUIRED_FIXTURES {
        let path = root.join(rel);
        assert!(
            path.is_file(),
            "missing fixture `{rel}` under tests/fixtures/ (grow SPEC only with fixtures)"
        );
    }
}

#[test]
fn spec_mentions_fixture_gate_paths() {
    let spec =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/SPEC.md"))
            .expect("SPEC.md");
    for needle in [
        "tests/fixtures/",
        "event-payload",
        "layout-shell",
        "io-child",
        "ng-content",
        "EventPayload",
    ] {
        assert!(
            spec.contains(needle),
            "SPEC.md must mention `{needle}` when that construct is in the growth contract"
        );
    }
}
