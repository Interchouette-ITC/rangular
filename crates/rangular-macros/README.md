# rangular-macros

Proc macros for rangular component templates.

Expands to `include!` of AOT output under `OUT_DIR/rangular/`. The consuming
crate's `build.rs` must call `rangular_aot::compile_named` first.
