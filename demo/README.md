# Browser demo

Leptos CSR app that AOT-compiles the full fixture corpus from `tests/fixtures/`:

**Components** (`tests/fixtures/components/`):

- `chrome-header` - bindings, `(click)`, keyboard `M`
- `color-field` - `@for`, inputs, palette
- `item-list` - `{{ }}`, `@for`
- `asset-icon` - `@if`, `[src]`, letter fallback
- `layout-shell` - `<rg-content>` projection
- `named-slots` - `<rg-content select>` + default slot
- `io-child` - `[label]` / `[muted]` in, `(muteToggle)` out

**HTML fixtures** (`tests/fixtures/html/`):

- `seed-bar` - seed input, Generate / Random (Random drives demo state)
- `pipes` - builtins (`uppercase`, `lowercase`, `number`, `json`) + demo custom `crab`
- `two-way` - `[(value)]` banana: type in the field (view → Host) or **Push from Host** (Host → view); mirror line shows Host state
- `field-required` - Host `required` validation + `nameDirty` (error after edit)
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
