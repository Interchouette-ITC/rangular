# rangular-host

Host binding API shared by AOT-generated views and the runtime interpreter
(events, inputs/outputs, validation helpers, for-scope implicits).

This is an implementation package. Applications should depend on the
[`rangular`](https://crates.io/crates/rangular) facade, which re-exports it as
`rangular::host`.
