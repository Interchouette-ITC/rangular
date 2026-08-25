use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let components = manifest.join("src/components");
    let css_out = manifest.join("style/components.generated.css");
    let out_dir = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("rangular");
    std::fs::create_dir_all(&out_dir).expect("create rangular OUT_DIR");

    println!("cargo:rerun-if-changed=src/components");

    // (dir under src/components, AOT fn name)
    let panels = [
        ("seed_bar", "seed_bar_view"),
        ("chrome_header", "chrome_header_view"),
        ("color_field", "color_field_view"),
        ("item_list", "item_list_view"),
        ("asset_icon", "asset_icon_view"),
        ("layout_shell", "layout_shell_view"),
        ("named_slots", "named_slots_view"),
        ("io_child", "io_child_view"),
        ("io_parent", "io_parent_view"),
        ("pipes", "pipes_view"),
        ("two_way", "two_way_view"),
        ("field_required", "field_required_view"),
        ("event_payload", "event_payload_view"),
        ("template_outlet", "template_outlet_view"),
    ];

    let mut css = String::from(
        "/* Generated from panel component SCSS - do not edit. */\n@layer components {\n",
    );
    for (dir, fn_name) in panels {
        compile_panel(&components, &out_dir, &mut css, dir, fn_name);
    }
    css.push_str("}\n");

    std::fs::write(&css_out, css).unwrap_or_else(|err| {
        panic!("write {}: {err}", css_out.display());
    });
}

fn compile_panel(components: &Path, out_dir: &Path, css: &mut String, dir: &str, fn_name: &str) {
    let panel_dir = components.join(dir);
    append_scss(css, dir, &panel_dir.join(format!("{dir}.scss")));

    let html_path = panel_dir.join(format!("{dir}.html"));
    let html = std::fs::read_to_string(&html_path).unwrap_or_else(|err| {
        panic!("read {}: {err}", html_path.display());
    });
    let source = format!("src/components/{dir}/{dir}.html");
    let aot = rangular_aot::compile_named(&html, &source, fn_name);
    assert!(aot.ok(), "{dir}.html: {:?}", aot.issues);
    let rs_path = out_dir.join(format!("{fn_name}.rs"));
    std::fs::write(&rs_path, &aot.code).unwrap_or_else(|err| {
        panic!("write {}: {err}", rs_path.display());
    });
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
