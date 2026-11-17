# rangular-aot

Compile-time lowering of rangular template IR to Leptos `view!` Rust source.

Used from `build.rs` (`compile_named`) before the consuming crate compiles.
Generated code refers to Rangular only through the `rangular` facade
(`rangular::host`, `rangular::aot`, `rangular::expr`), so consuming crates must
depend on `rangular`.

This is an implementation package. Applications should depend on the
[`rangular`](https://crates.io/crates/rangular) facade, which re-exports it as
`rangular::aot`.
