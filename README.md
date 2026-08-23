# rangular

Write Angular-shaped templates as external `.html` and `.scss`, keep state and
handlers in Rust, and render with [Leptos](https://leptos.dev/) in the browser
(CSR / Trunk / wasm).

Production builds use **AOT** lowering to Leptos. A **runtime** interpreter
shares the same AST for parity tests and tooling.

## Status

| Area                 | Today                                                                    |
| -------------------- | ------------------------------------------------------------------------ |
| Language             | v0.1 contract in [`docs/SPEC.md`](docs/SPEC.md)                          |
| Parser / expr / host | Core subset                                                              |
| AOT                  | `rangular-aot` + `rangular-macros`                                       |
| Runtime              | `rangular-runtime` (parity / tooling)                                    |
| SCSS                 | `rangular-css` (flat `compile_scss`, or encapsulate; Bootstrap utilities stay global) |
| Registry             | Panel tags + typed `provide` / `inject`                                  |
| Growth               | Fixture corpus in [`tests/fixtures/`](tests/fixtures/)                   |

Honest subset, not full Angular. New syntax lands through fixtures and semver.
Unsupported input yields `RANG*` diagnostics; templates must never panic the process.

## Goals

| Goal               | Approach                                               |
| ------------------ | ------------------------------------------------------ |
| Markup out of Rust | External templates; no panel `view!` in controllers    |
| Familiar authoring | Angular-shaped bindings, `@if` / `@for`, component CSS |
| Web-native         | [Leptos](https://leptos.dev/) CSR on wasm              |
| Safe parsing       | Diagnostics on bad input; no panic on template text    |

Other Rust + Angular-template projects often target native or hybrid desktop
GUIs. rangular targets the **browser DOM**, with a versioned subset and tests.

## Roadmap

- Keep AOT and runtime aligned on the fixture corpus
- Grow the subset only when fixtures land
- Content projection / slots for layout shells
- Stronger host typing and event payloads
- crates.io when the v0.1 surface is stable

## Quick start

```bash
make check   # cargo check --workspace
make lint    # fmt + clippy
make test    # workspace tests (includes parity)
make ci      # lint + test + no-panic garbage fixtures
```

How to add fixtures: [`tests/fixtures/README.md`](tests/fixtures/README.md) and
[`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md).

## Workspace

| Crate              | Role                                         |
| ------------------ | -------------------------------------------- |
| `rangular-parser`  | HTML + Angular syntax → AST                  |
| `rangular-expr`    | Expression AST and evaluation                |
| `rangular-css`     | Component SCSS, `:host`, encapsulation       |
| `rangular-host`    | get / set / call / events                    |
| `rangular-aot`     | Compile-time lowering to Leptos              |
| `rangular-macros`  | `rangular_template!` (includes build output) |
| `rangular-runtime` | Interpret AST at runtime                     |
| `rangular`         | Facade + panel registry                      |

## Docs

- [`docs/SPEC.md`](docs/SPEC.md) - language contract
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) - fixtures and PR habits

## License

MIT. See [`LICENSE`](LICENSE).
