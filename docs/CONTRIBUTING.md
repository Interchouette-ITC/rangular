# Contributing to rangular

## Fixture-driven language growth

**[rangular](https://github.com/Interchouette-ITC/rangular)** evolves from
**`tests/fixtures/`**. Each fixture is a small Angular-shaped
HTML and/or SCSS example that parser, SCSS, AOT, and runtime tests must handle or
document as unsupported.

### Add a component fixture

1. Create `tests/fixtures/components/<name>/`.
2. Add `<name>.html` and `<name>.scss`.
3. Use syntax you want in the contract: `{{ }}`, `[prop]`, `(event)`, `@if`, `@for`,
   `:host`, `[class.foo]`, `[attr.aria-*]`, SCSS nesting, and so on.
4. Run `make test` (and `make no-panic` after parser changes).
5. Document the path in `docs/SPEC.md` and add it to
   `REQUIRED_FIXTURES` in `crates/rangular-parser/tests/fixtures_gate.rs` so CI
   fails if the fixture disappears.

### Unsupported syntax

If a fixture uses syntax **outside** the current `SPEC.md` version:

1. Extend parser/CSS/backends to support it, **or**
2. Add a golden test that expects a diagnostic code from the `RANG*` series.

**Never** panic or abort the process on template content. See the best-effort
policy in `SPEC.md`.

### App templates stay in apps

Product UI templates belong in the consuming application repository. The
**[rangular](https://github.com/Interchouette-ITC/rangular)** corpus holds engine
examples only.

## Depend from a local clone or pin a commit

Applications depend on the `rangular` facade and the `rangular-build` build
helper only. For unreleased changes, pin a commit:

```toml
[dependencies]
rangular = { git = "https://github.com/Interchouette-ITC/rangular.git", rev = "…" }

[build-dependencies]
rangular-build = { git = "https://github.com/Interchouette-ITC/rangular.git", rev = "…" }
```

For day-to-day hacking against a sibling checkout:

```toml
[dependencies]
rangular = { path = "../rangular/crates/rangular" }

[build-dependencies]
rangular-build = { path = "../rangular/crates/rangular-build" }
```

Adjust the relative path to match your layout.

## Code changes

- Prefer one concern per PR.
- The root `Makefile` only includes `make/*.mk` (`make help` lists targets).
  Put new targets in the matching `make/` file, and move any recipe longer
  than a few lines into a sibling `.sh` script, as `make/version/` does.
- Run `make lint` and `make test` before push. Changes to code generation or
  rendering should also pass `make test-hydration` (needs `wasm32-unknown-unknown`,
  `wasm-bindgen-test-runner` matching the consumer fixture lockfile, and
  chromedriver; set `CHROME_BIN` if your default Chrome does not match it).
- Generated code must reference Rangular only through `rangular::…` facade
  paths; `crates/rangular-aot/tests/smoke.rs` and the consumer fixture under
  `crates/rangular-build/tests/fixtures/document-consumer/` enforce this.
- Local fuzz / unsafe audit: `make fuzz` (default target `parse-html`, needs nightly +
  `cargo fuzz`) and `make geiger` (needs `cargo geiger`).
- Conventional commits: `feat(parser): …`, `fix(aot): …`, etc.
- PR body follows [`pull_request_template.md`](pull_request_template.md)
  (**Summary** + **Test plan** only).
- Follow the [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) and
  [`SECURITY.md`](SECURITY.md).

## Questions

Use GitHub issue forms (Bug report / Feature request) when opening an issue in
this repository.
