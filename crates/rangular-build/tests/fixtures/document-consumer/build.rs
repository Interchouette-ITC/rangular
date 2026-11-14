use std::{env, path::PathBuf};

fn main() {
    // Absolute path, so the CSS goes to OUT_DIR and not the source tree.
    let css = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("site.css");
    rangular_build::build(
        rangular_build::Config::new("src/components", css)
            .document("src/document/document.html")
            .head("src/document/head.html")
            .body("src/document/body.html"),
    );
}
