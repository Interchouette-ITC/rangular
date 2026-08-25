# Test fixtures (how to add examples)

This directory is the **corpus** that drives
**[rangular](https://github.com/Interchouette-ITC/rangular)** language growth.

## What this is

- Small **Angular-shaped** `.html` + component `.scss` files used by parser /
  SCSS / AOT / runtime tests.
- They are **engine examples**, not the browser demo UI and not product UI.
  Apps (and `demo/`) keep their own templates in their own trees.

## Layers

| Layer | Role |
| ----- | ---- |
| `tests/fixtures/` | Template / SCSS examples for language growth and crate tests |
| `demo/` | Browser app that dogfoods rangular (independent panels) |
| Product apps | Separate consumers; out of this tree |

No symlinks or path coupling between `demo/` and `tests/fixtures/`.

## How to add a fixture

1. Create a folder under `tests/fixtures/components/<name>/`.
2. Add `<name>.html` and `<name>.scss` (minimal `:host` stub is fine).
3. Prefer real Angular patterns you want supported: `{{ }}`, `[prop]`, `(event)`,
   `@if` / `@for`, `:host`, SCSS nesting, `rg-content` / `ng-content` / `select`, pipes (`|`),
   banana (`[(…)]`), `ng-template` / outlet, etc.
4. If the syntax is **not yet in SPEC v0.x**, either:
   - extend the parser/SCSS/backends to support it, **or**
   - add an expected-diagnostics golden that documents "unsupported" (must
     **not** panic).
5. Name the path in [`docs/SPEC.md`](../../docs/SPEC.md) and in the
   `REQUIRED_FIXTURES` list in `crates/rangular-parser/tests/fixtures_gate.rs`.
6. Run the crate tests that consume the corpus (`make test`, `make no-panic`).

Hosts for runtime tests stay **inline** in crate tests (no Host `.rs` under
fixtures).

## Layout

```text
tests/fixtures/
  components/
    seed-bar/            # seed input, Generate / Random
    event-payload/       # typed $event / EventPayload
    io-parent/           # parent↔child IO teaching shape
    pipes/               # {{ value | pipe }} builtins
    two-way/             # [(value)] banana Host get/set
    field-required/      # banana + Host required / error text
    template-outlet/     # ng-template #ref + [ngTemplateOutlet]
    item-list/           # simple @for list
    chrome-header/       # [attr], [class], (click), {{ }}
    color-field/         # @for, [disabled], (click), :host
    asset-icon/          # @if / @else, [src], :host
    layout-shell/        # rg-content projection slot
    named-slots/         # rg-content select=".header" + default
    io-child/            # child surface for [label] / mute output
  html/                  # reserved for one-off minimal HTML (gitkeep)
  scss/                  # reserved for one-off SCSS (gitkeep)
  css/
  README.md              # this file
```
