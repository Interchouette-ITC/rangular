# Browser demo

Leptos CSR app that AOT-compiles fixture templates from `tests/fixtures/`:

- `html/seed-bar` - inputs, `[disabled]`, `(click)`
- `components/item-list` - `{{ }}`, `@for`

## Run locally

```bash
make demo
# → http://127.0.0.1:4180/
```

Release build:

```bash
make demo-build
# dist/ under this directory
```

Port override: `make demo DEMO_PORT=3000`.

## Docker (Render / Hub)

Same wasm SPA as the browser demo, served by nginx:

```bash
make docker-build-dev
make docker-run
# → http://127.0.0.1:8080/
```

Production URL: https://rangular.interchouette.net/

Pull:

```bash
docker pull interchouette/rangular-demo:dev
```

## Lint

Demo is outside the default workspace clippy path:

```bash
make demo-check
```
