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
   `@if` / `@for`, `:host`, SCSS nesting, etc.
4. If the syntax is **not yet in SPEC v0.x**, either:
   - extend the parser/SCSS/backends to support it, **or**
   - add an expected-diagnostics golden that documents "unsupported" (must
     **not** panic).
5. Run the crate tests that consume the corpus (`make test`, `make no-panic`).

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
  scss/              # minimal SCSS / :host / Bootstrap coexist reproducers
  components/
    item-list/       # simple @for list (teaching example)
    chrome-header/   # [attr], [class], (click), {{ }}
    color-field/     # @for, [disabled], (click), :host
    asset-icon/      # @if / @else, [src], :host
  README.md          # this file
```
