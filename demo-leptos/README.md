# Browser demo (Leptos)

Leptos CSR app that dogfoods rangular with colocated panels under
`demo-leptos/src/components/<name>/` (`html` + `scss` + `rs`). The demo does
**not** compile `tests/fixtures/`; that corpus is for crate tests only.

## Panels

Each feature panel is its own folder:

| Panel             | Notes                                                            |
| ----------------- | ---------------------------------------------------------------- |
| `seed_bar`        | AOT seed input, Generate / Random (drives shared tick)           |
| `chrome_header`   | bindings, `(click)`, keyboard `M`                                |
| `color_field`     | `@for`, inputs, palette                                          |
| `item_list`       | `{{ }}`, `@for`                                                  |
| `asset_icon`      | `@if`, `[src]`, letter fallback                                  |
| `layout_shell`    | `<rg-content>` projection                                        |
| `named_slots`     | `<rg-content select>` + default slot                             |
| `io_child`        | `[label]` / `[muted]` in, `(muteToggle)` out                     |
| `io_parent`       | AOT heading + live AOT `io_child` (nested tags are not live yet) |
| `pipes`           | builtins + demo custom `crab`                                    |
| `two_way`         | `[(value)]` banana                                               |
| `field_required`  | Host `required` + dirty error                                    |
| `event_payload`   | typed `$event` / `EventPayload`                                  |
| `template_outlet` | `ng-template` + `[ngTemplateOutlet]`                             |

Page chrome (`app.rs` / `decor.rs`) stays plain Leptos.

## Code showcase

A floating **CODE** rail (`src/showcase/`) embeds each panel’s colocated
`.html` / `.scss` / `.rs` via `include_str!` and presents them in a read-only
editor. Hovering a fixture panel (or following a hash link) selects its sources;
the selection stays until another panel is hovered. On viewports ≤1024px the
rail defaults to collapsed and expands as a bottom sheet.

`style/demo-swatches.css` is a demo-only workaround: swatch paints keyed off
`data-swatch` (dynamic style binding is not available yet) plus small decor
overrides. Panel look lives in each panel `.scss`.

Use **Generate** / **Random** on the seed bar to cycle panel state.

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

Production URL: [https://rangular.interchouette.net](https://rangular.interchouette.net)

Desktop installers: **[GitHub Releases](https://github.com/Interchouette-ITC/rangular/releases)** (Linux `.deb` / AppImage, Windows NSIS).

Pull:

```bash
docker pull interchouette/rangular-demo:latest
```

## Desktop (Tauri)

Same SPA in a native webview:

```bash
make demo-tauri
```

Details: [`../demo-tauri/README.md`](../demo-tauri/README.md).

## Lint

Demo is outside the default workspace clippy path:

```bash
make demo-check
```
