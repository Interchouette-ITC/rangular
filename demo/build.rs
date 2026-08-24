use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = manifest.join("../tests/fixtures");
    let css_out = manifest.join("style/components.generated.css");
    let out_dir = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("rangular");
    std::fs::create_dir_all(&out_dir).expect("create rangular OUT_DIR");

    println!("cargo:rerun-if-changed={}", fixture_root.display());

    let mut css = String::from("/* Generated from fixture SCSS - do not edit. */\n");

    let components = [
        ("chrome-header", "chrome_header_view"),
        ("color-field", "color_field_view"),
        ("item-list", "item_list_view"),
        ("asset-icon", "asset_icon_view"),
        ("layout-shell", "layout_shell_view"),
        ("named-slots", "named_slots_view"),
        ("io-child", "io_child_view"),
    ];

    for (name, fn_name) in components {
        compile_component(
            &fixture_root,
            &out_dir,
            &mut css,
            name,
            fn_name,
            &format!("tests/fixtures/components/{name}/{name}.html"),
        );
    }

    let html_fixtures = [
        ("pipes.html", "pipes_view", None),
        ("two-way.html", "two_way_view", None),
        ("field-required.html", "field_required_view", None),
        ("event-payload.html", "event_payload_view", None),
        ("template-outlet.html", "template_outlet_view", None),
        ("seed-bar.html", "seed_bar_view", Some("scss/seed-bar-coexist.scss")),
        ("io-parent.html", "io_parent_view", None),
    ];

    for (html_name, fn_name, scss_rel) in html_fixtures {
        let html_path = fixture_root.join("html").join(html_name);
        let html = std::fs::read_to_string(&html_path).unwrap_or_else(|err| {
            panic!("read {}: {err}", html_path.display());
        });
        let source = format!("tests/fixtures/html/{html_name}");
        emit_aot(&out_dir, &html, &source, fn_name);

        if let Some(scss_rel) = scss_rel {
            let scss_path = fixture_root.join(scss_rel);
            append_scss(&mut css, html_name.trim_end_matches(".html"), &scss_path);
        }
    }

    std::fs::write(&css_out, css).unwrap_or_else(|err| {
        panic!("write {}: {err}", css_out.display());
    });
}

fn compile_component(
    fixture_root: &Path,
    out_dir: &Path,
    css: &mut String,
    name: &str,
    fn_name: &str,
    source: &str,
) {
    let panel_dir = fixture_root.join("components").join(name);
    let scss_path = panel_dir.join(format!("{name}.scss"));
    append_scss(css, name, &scss_path);

    let html_path = panel_dir.join(format!("{name}.html"));
    let html = std::fs::read_to_string(&html_path).unwrap_or_else(|err| {
        panic!("read {}: {err}", html_path.display());
    });
    emit_aot(out_dir, &html, source, fn_name);
}

fn append_scss(css: &mut String, label: &str, scss_path: &Path) {
    let scss = std::fs::read_to_string(scss_path).unwrap_or_else(|err| {
        panic!("read {}: {err}", scss_path.display());
    });
    let result = rangular_css::compile_scss(&scss);
    assert!(result.ok(), "{label}.scss: {:?}", result.issues);
    let _ = write!(css, "\n/* --- {label} --- */\n");
    css.push_str(&result.css);
    css.push('\n');
}

fn emit_aot(out_dir: &Path, html: &str, source: &str, fn_name: &str) {
    let aot = rangular_aot::compile_named(html, source, fn_name);
    assert!(aot.ok(), "{source}: {:?}", aot.issues);
    let rs_path = out_dir.join(format!("{fn_name}.rs"));
    std::fs::write(&rs_path, &aot.code).unwrap_or_else(|err| {
        panic!("write {}: {err}", rs_path.display());
    });
}
