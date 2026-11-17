# rangular-runtime

Runtime interpreter for rangular templates (parity path alongside AOT/Leptos).

Useful for tests and hosts that evaluate IR without a full wasm build.

This is an implementation package. Applications should depend on the
[`rangular`](https://crates.io/crates/rangular) facade, which re-exports it as
`rangular::runtime`.
