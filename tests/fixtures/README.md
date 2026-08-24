# Test fixtures (how to add examples)

This directory is the **corpus** that drives
**[rangular](https://github.com/Interchouette-ITC/rangular)** language growth.

## What this is

- Small **Angular-shaped** `.html` + component `.scss` files used by parser /
  SCSS / AOT / runtime tests.
- They are **engine examples**, not product UI. Apps keep their own templates
  in the app repo.

## How to add a fixture

1. Create a folder under `tests/fixtures/components/<name>/`.
2. Add `<name>.html` and `<name>.scss`.
3. Prefer real Angular patterns you want supported: `{{ }}`, `[prop]`, `(event)`,
   `@if` / `@for`, `:host`, SCSS nesting, `ng-content` / `select`, pipes (`|`),
   banana (`[(…)]`), `ng-template` / outlet, etc.
4. If the syntax is **not yet in SPEC v0.x**, either:
   - extend the parser/SCSS/backends to support it, **or**
   - add an expected-diagnostics golden that documents "unsupported" (must
     **not** panic).
5. Name the path in [`docs/SPEC.md`](../../docs/SPEC.md) and in the
   `REQUIRED_FIXTURES` list in `crates/rangular-parser/tests/fixtures_gate.rs`.
6. Run the crate tests that consume the corpus (`make test`, `make no-panic`).

## Flat html/scss samples

One-off files under:

- `tests/fixtures/html/`
- `tests/fixtures/scss/`

Use these for minimal reproducers (single binding, one `:host` rule) without a
full component folder.

## Layout

```text
tests/fixtures/
  html/              # minimal single-file HTML reproducers
    seed-bar.html
    event-payload.html   # typed $event / EventPayload
    io-parent.html       # parent↔child IO teaching shape
    pipes.html           # {{ value | pipe }} builtins
    two-way.html         # [(value)] banana Host get/set
    field-required.html  # banana + Host required / error text
    template-outlet.html # ng-template #ref + [ngTemplateOutlet]
  scss/              # minimal SCSS / :host / Bootstrap coexist reproducers
  components/
    item-list/       # simple @for list (teaching example)
    chrome-header/   # [attr], [class], (click), {{ }}
    color-field/     # @for, [disabled], (click), :host
    asset-icon/      # @if / @else, [src], :host
    layout-shell/    # ng-content projection slot
    named-slots/     # ng-content select=".header" + default
    io-child/        # child surface for [label] / mute output pattern
  README.md          # this file
```
