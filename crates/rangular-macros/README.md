# rangular-macros

Proc macros for rangular component templates.

Expands to `include!` of AOT output under `OUT_DIR/rangular/`. The consuming
crate's `build.rs` must generate that file first, normally with
`rangular-build` (or `rangular::aot::compile_named` for custom build scripts).

This is an implementation package. Applications should depend on the
[`rangular`](https://crates.io/crates/rangular) facade, which re-exports it as
`rangular::macros`.
