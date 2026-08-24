# Browser demo

Leptos CSR app that AOT-compiles the full fixture corpus from `tests/fixtures/`:

**Components** (`tests/fixtures/components/`):

- `chrome-header` - bindings, `(click)`, keyboard `M`
- `color-field` - `@for`, inputs, palette
- `item-list` - `{{ }}`, `@for`
- `asset-icon` - `@if`, `[src]`, letter fallback
- `layout-shell` - `<ng-content>` projection
- `named-slots` - `<ng-content select>` + default slot
- `io-child` - child IO surface (used inside `io-parent` panel)

**HTML fixtures** (`tests/fixtures/html/`):

- `seed-bar` - seed input, Generate / Random (Random drives demo state)
- `pipes` - `{{ value | pipe }}`
- `two-way` - `[(value)]` banana binding
- `field-required` - Host `required` validation
- `event-payload` - typed `$event` / `EventPayload`
- `template-outlet` - `ng-template` + `[ngTemplateOutlet]`
- `io-parent` - parent IO shape (live child wired in demo host)

Use **Random** on the seed bar (or fixture panels reacting to the shared tick) to cycle demo state.

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
