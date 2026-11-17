# rangular

Angular-shaped HTML/SCSS templates for [Leptos](https://leptos.dev/): browser
CSR, native SSR, and SSR with browser hydration.

`rangular` is the public entry point. Applications depend on it plus the
[`rangular-build`](https://crates.io/crates/rangular-build) build helper; the
other workspace crates are implementation details reached through these
modules:

| Module              | Contents                                                      |
| ------------------- | ------------------------------------------------------------- |
| `rangular::host`    | `Host`, `HostError`, `Value`, `EventPayload`, validators      |
| `rangular::aot`     | `HostCell` for generated views; `compile_named` for custom builds |
| `rangular::expr`    | Expressions and `PipeRegistry`                                |
| `rangular::css`     | `compile_scss`, `encapsulate`                                 |
| `rangular::macros`  | `rangular_template!`                                          |
| `rangular::parser`  | Template parser and diagnostics                               |
| `rangular::runtime` | Runtime interpreter for tests and tooling                     |

Generated views reference Rangular only through `rangular::…` paths.
`rangular-build` is not re-exported: build scripts need it in
`[build-dependencies]`.

## Example

`Cargo.toml`:

```toml
[dependencies]
leptos = { version = "0.8", features = ["csr"] } # "ssr" on the server, "hydrate" in the browser
rangular = "1.0.6"
# Needed because generated event handlers name `web_sys::MouseEvent` and friends.
web-sys = { version = "0.3", features = ["Event", "MouseEvent"] }

[build-dependencies]
rangular-build = "1.0.6"
```

`build.rs`:

```rust,ignore
fn main() {
    rangular_build::build(rangular_build::Config::new(
        "src/components",
        "style/components.css",
    ));
}
```

`src/components/counter/counter.html` (an optional `counter.scss` next to it is
compiled into `style/components.css`):

```html
<section class="counter">
  <p>Count: {{ count }}</p>
  @if (positive) {
  <p>positive</p>
  } @else {
  <p>zero</p>
  }
  <button type="button" (click)="increment()">Increment</button>
</section>
```

`src/main.rs`:

```rust,ignore
use leptos::prelude::*;
use rangular::aot::HostCell;
use rangular::host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/counter_view.rs"));

#[derive(Clone, Copy)]
struct Counter {
    count: RwSignal<u32>,
}

impl Host for Counter {
    fn get(&self, name: &str) -> Option<Value> {
        match name {
            "count" => Some(Value::Num(f64::from(self.count.get()))),
            "positive" => Some(Value::Bool(self.count.get() > 0)),
            _ => None,
        }
    }

    fn call(&mut self, name: &str, _: &[Value]) -> Result<Value, HostError> {
        if name == "increment" {
            self.count.update(|count| *count += 1);
        }
        Ok(Value::Unit)
    }
}

fn app() -> impl IntoView {
    counter_view(HostCell::new(Counter { count: RwSignal::new(0) }))
}

fn main() {
    // Browser CSR. For SSR render `app().to_html()` on the server and call
    // `leptos::mount::hydrate_body(app)` in the browser instead.
    leptos::mount::mount_to_body(app);
}
```

Link `style/components.css` from your `index.html` (Trunk:
`<link data-trunk rel="css" href="style/components.css" />`).

## Documentation

- [Guide](https://github.com/Interchouette-ITC/rangular/blob/HEAD/docs/README.md)
- [Language specification](https://github.com/Interchouette-ITC/rangular/blob/HEAD/docs/SPEC.md)
- [Changelog](https://github.com/Interchouette-ITC/rangular/blob/HEAD/docs/CHANGELOG.md)

## License

Apache-2.0. See [LICENSE](https://github.com/Interchouette-ITC/rangular/blob/HEAD/LICENSE).
