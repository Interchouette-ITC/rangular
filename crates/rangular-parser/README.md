# rangular-parser

Angular-subset HTML template parser for rangular.

Produces IR consumed by AOT lowering and the runtime interpreter. Fixture
corpus lives under `tests/fixtures/`.

This is an implementation package. Applications should depend on the
[`rangular`](https://crates.io/crates/rangular) facade, which re-exports it as
`rangular::parser`.
