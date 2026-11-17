# rangular-build

Cargo build-script support for Rangular. Add it as a **build dependency** next
to the `rangular` facade:

```toml
[dependencies]
leptos = "0.8"
rangular = "1.1.0"

[build-dependencies]
rangular-build = "1.1.0"
```

```rust,no_run
// build.rs
fn main() {
    rangular_build::build(rangular_build::Config::new(
        "src/components",
        "style/components.css",
    ));
}
```

Each `src/components/<name>/<name>.html` compiles to
`OUT_DIR/rangular/<name>_view.rs`; matching `<name>.scss` files are compiled
into the configured stylesheet. Optional document, head, and body templates
are registered with `Config::document`, `Config::head`, and `Config::body`.

Generated views reference Rangular only through `rangular::…` paths, so the
consuming crate needs `rangular` as a normal dependency. `rangular-build` is a
separate package and is not re-exported by the facade.

See the [Rangular guide](https://github.com/Interchouette-ITC/rangular/blob/HEAD/docs/README.md).

## License

Apache-2.0. See [LICENSE](https://github.com/Interchouette-ITC/rangular/blob/HEAD/LICENSE).
