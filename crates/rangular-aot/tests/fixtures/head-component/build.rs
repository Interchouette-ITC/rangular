use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=head.html");
    let adapter = rangular_aot::HeadComponentAdapter::new(
        "serenade_view::ManagedTitle",
        "serenade_view::ManagedMeta",
        "serenade_view::ManagedLink",
    );
    let result = rangular_aot::compile_head_named_with_adapter(
        include_str!("head.html"),
        "head.html",
        "generated_head",
        &adapter,
    );
    assert!(result.ok(), "head compilation failed: {:?}", result.issues);

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("generated_head.rs");
    fs::write(output, result.code).expect("write generated head component");
}
