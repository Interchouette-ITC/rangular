use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = manifest.join("../tests/fixtures");
    let css_out = manifest.join("style/components.generated.css");
    let out_dir = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("rangular");
    std::fs::create_dir_all(&out_dir).expect("create rangular OUT_DIR");

    println!("cargo:rerun-if-changed={}", fixture_root.display());

    let panels: [(&str, &str, &str); 2] = [
        ("components/item-list", "item-list", "item_list_view"),
        ("html", "seed-bar", "seed_bar_view"),
    ];

    let mut css = String::from("/* Generated from fixture SCSS - do not edit. */\n");
    for (dir, name, fn_name) in panels {
        let panel_dir = fixture_root.join(dir);
        let scss_path = panel_dir.join(format!("{name}.scss"));
        if scss_path.exists() {
            let scss = std::fs::read_to_string(&scss_path).unwrap_or_else(|err| {
                panic!("read {}: {err}", scss_path.display());
            });
            let result = rangular_css::compile_scss(&scss);
            assert!(result.ok(), "{name}.scss: {:?}", result.issues);
            let _ = write!(css, "\n/* --- {name} --- */\n");
            css.push_str(&result.css);
            css.push('\n');
        }

        let html_path = panel_dir.join(format!("{name}.html"));
        let html = std::fs::read_to_string(&html_path).unwrap_or_else(|err| {
            panic!("read {}: {err}", html_path.display());
        });
        let source = format!("tests/fixtures/{dir}/{name}.html");
        let aot = rangular_aot::compile_named(&html, &source, fn_name);
        assert!(aot.ok(), "{dir}.html: {:?}", aot.issues);
        let rs_path = out_dir.join(format!("{fn_name}.rs"));
        std::fs::write(&rs_path, &aot.code).unwrap_or_else(|err| {
            panic!("write {}: {err}", rs_path.display());
        });
    }

    std::fs::write(&css_out, css).unwrap_or_else(|err| {
        panic!("write {}: {err}", css_out.display());
    });
}
