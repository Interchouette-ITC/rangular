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

Until crates.io, the README shows a `git` + `branch = "dev"` dependency. To
freeze a tree, pin a commit:

```toml
rangular-aot = { git = "https://github.com/Interchouette-ITC/rangular.git", rev = "…" }
rangular-css = { git = "https://github.com/Interchouette-ITC/rangular.git", rev = "…" }
rangular-host = { git = "https://github.com/Interchouette-ITC/rangular.git", rev = "…" }
```

For day-to-day hacking against a sibling checkout:

```toml
rangular-aot = { path = "../rangular/crates/rangular-aot" }
rangular-css = { path = "../rangular/crates/rangular-css" }
rangular-host = { path = "../rangular/crates/rangular-host" }
```

Adjust the relative path to match your layout.

## Code changes

- Prefer one concern per PR.
- Run `make lint` and `make test` before push.
- Conventional commits: `feat(parser): …`, `fix(aot): …`, etc.

## Questions

Open an issue in this repository once it is published.
