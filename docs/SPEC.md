# [**rangular**](https://github.com/Interchouette-ITC/rangular) specification v0.1

This document is the language contract for
**[rangular](https://github.com/Interchouette-ITC/rangular)** **0.1.x**. Patch
releases clarify diagnostics and fix bugs without breaking supported syntax.
Minor releases may add constructs when fixtures and tests cover them. Major
releases may remove or rename supported constructs only with a migration note.

## Scope

| In scope                                       | Out of scope (v0.1)             |
| ---------------------------------------------- | ------------------------------- |
| Web DOM via Leptos CSR                         | Full Angular framework          |
| External `.html` templates                     | Markup inside Rust controllers  |
| Component `.scss` (compiled then encapsulated) | Pipes, i18n, NgModule           |
| AOT (production default)                       | Claiming runtime parity in prod |
| Runtime interpreter (tests/tooling)            | Native desktop GUI toolkits     |
| Fixture-driven growth                          | Silent ignore of unknown syntax |

**Desktop shells with a webview** (for example Tauri) are still the web path:
the UI is Leptos CSR / wasm inside the webview. They are **not** a separate
native-widget target. For Angular-style **native** desktop GUIs, see external
projects such as Angust; they are outside this contract.

## Authoring model

```text
Controller (Rust)     state, services, handlers - no markup
Component shell       inputs/outputs, inject, template path
Template (*.html)     Angular-subset markup
Styles (*.scss)       component rules + :host
App global CSS        loaded by the host app (not rewritten by rangular)
```

Global utility classes (for example `.btn`, `.container`) come from the host
application stylesheet. **[rangular](https://github.com/Interchouette-ITC/rangular)**
does **not** ship or require a particular global CSS framework. Component CSS
encapsulation must not mangle global selectors the app loads separately.

## Template grammar (v0.1 core)

### Elements and text

- Normal HTML elements and static text nodes.
- `{`{ expression `}`}` for text interpolation.
- HTML comments are preserved in the AST; they do not affect bindings.

### Property and attribute bindings

| Surface syntax        | Meaning                     |
| --------------------- | --------------------------- |
| `[prop]="expr"`       | DOM property binding        |
| `[attr.name]="expr"`  | Attribute binding           |
| `[class.name]="expr"` | Toggle CSS class on element |

Static attributes without brackets pass through unchanged.

### Event bindings

| Surface syntax              | Meaning                                 |
| --------------------------- | --------------------------------------- |
| `(click)="handler($event)"` | DOM event; `$event` is the event object |
| `(input)="onInput($event)"` | Common form events                      |

Handler names resolve through the host `call` API.

### Control flow

| Construct                               | Notes                           |
| --------------------------------------- | ------------------------------- |
| `@if (cond) { … }`                      | Primary v0.1 conditional        |
| `@else { … }`                           | Optional else block             |
| `@for (item of list; track item) { … }` | Primary v0.1 loop               |
| `*ngIf="cond"`                          | Desugars to `@if` during parse  |
| `*ngFor="let x of xs"`                  | Desugars to `@for` during parse |

### Expressions (v0.1)

Literals (string, number, bool), identifiers and dotted paths, unary `!`,
binary `&&`, `||`, `==`, `!=`, parenthesis, and handler calls such as
`onGenerate()` or `seedChange($event)`.

**Not in v0.1:** pipes, ternary, nullish coalescing, assignment, arbitrary
method chains beyond what the host exposes.

## Reference template: seed bar

Illustrative panel markup (lives in consuming apps, not in this repo corpus):

```html
<section class="seed-bar" aria-label="Seed controls">
  <label for="seed">Seed</label>
  <input
    id="seed"
    type="text"
    [value]="seed"
    (input)="seedChange($event)"
    spellcheck="false"
    autocomplete="off"
  />
  <button
    class="btn btn-primary"
    type="button"
    [disabled]="generateDisabled"
    (click)="onGenerate()"
  >
    Generate
  </button>
  <button
    class="btn btn-secondary"
    type="button"
    [disabled]="randomDisabled"
    (click)="onRandom()"
  >
    Random
  </button>
</section>
```

The controller exposes `seed`, `generateDisabled`, `randomDisabled`, and the
handler methods. Class names such as `btn` and `btn-primary` are **global** app
CSS; rangular passes them through on the element unchanged.

## Component SCSS (v0.1)

Authors write **SCSS** component sheets (`:host`, nesting, `&`). `rangular_css::encapsulate`
compiles them (grass) then applies emulated encapsulation. Output is flat CSS for
the browser. Encapsulation is **emulated**: scoped rules receive a generated
attribute selector; `:host` rewrites to that host selector.

Example (from the `color-field` fixture):

```scss
:host {
  display: block;
  pointer-events: none;
}

.color-field__label {
  display: block;
  font-size: 0.75rem;
}
```

Rules:

- `:host` and `:host(...)` rewrite to the component host attribute selector.
- Class and element selectors defined in the component sheet are scoped with a
  content attribute on the rightmost compound selector.
- Selectors that are **only** known global utility classes (for example
  `.btn`, `.btn-primary`, `.container`) are **not** scoped, so host-app
  Bootstrap (or similar) sheets keep matching.
- Unbalanced or malformed input yields `RANG301` diagnostics; never panics.

API: `rangular_css::encapsulate` (SCSS) with `ScopeAttrs`. Flat CSS helpers are
test/tooling only (`encapsulate_css`).

## Host API (shared by AOT and runtime)

| Operation          | Purpose                                              |
| ------------------ | ---------------------------------------------------- |
| `get(path)`        | Read a binding value (`seed`, `generateDisabled`, …) |
| `set(path, value)` | Two-way / input side effects when needed             |
| `call(name, args)` | Invoke controller handlers                           |
| Event bridge       | Map `(click)` etc. to host callables                 |

## Inject / registry

Typed services use `Registry::provide` / `Registry::inject` (TypeId map). Panel
custom tags resolve through the same registry:

| Tag               | Component        |
| ----------------- | ---------------- |
| `app-root`        | AppRoot          |
| `app-site-header` | SiteHeader       |
| `app-seed-bar`    | SeedBar          |
| `app-preview`     | PreviewPanel     |
| `app-accessories` | AccessoriesPanel |

`Registry::with_default_panels()` registers those tags. At the Leptos app edge,
apps map this to `provide_context` / `use_context` as needed.

## Diagnostics

Format:

```text
error[RANG001]: message at path:line:col
warning[RANG101]: message at path:line:col
```

| Code range          | Kind                                       |
| ------------------- | ------------------------------------------ |
| `RANG001`–`RANG099` | Parse errors (fatal for that template)     |
| `RANG101`–`RANG199` | Parse warnings (unsupported but non-fatal) |
| `RANG201`–`RANG299` | Expression errors                          |
| `RANG301`–`RANG399` | CSS errors                                 |
| `RANG401`–`RANG499` | AOT lowering errors                        |
| `RANG501`–`RANG599` | Runtime interpret errors                   |

Examples:

- `RANG001` unexpected token in template
- `RANG101` unknown directive (skipped when safe)
- `RANG201` unknown identifier in expression
- `RANG301` malformed `:host` selector
- `RANG501` empty template at runtime

AOT may promote selected warnings to compile errors when configured; default
builds fail on `RANG001` and `RANG201` for in-scope fixtures.

Production builds use **AOT**. The **runtime** interpreter shares the same AST
and Host API for parity tests and tooling; it is not the production path.

## Best-effort / no-panic policy

Template and CSS input must **never** cause `panic!`, `unwrap` on user content,
or process abort.

| Situation                      | Behavior                                                                           |
| ------------------------------ | ---------------------------------------------------------------------------------- |
| Unknown attribute or directive | Emit `RANG101` (or structured skip); continue when safe                            |
| Malformed but bounded input    | Return `Err` with diagnostics list                                                 |
| Garbage / fuzz input           | Return diagnostics; no stack unwind                                                |
| Internal invariant broken      | Allowed to panic only on programmer bugs in rangular itself, not on user templates |

Parser, runtime, and AOT share diagnostic types so fixtures can golden-test
both success and expected unsupported cases.

## Fixture-driven expansion

1. Add or update a file under `tests/fixtures/`.
2. If syntax is new, bump the **minor** spec version and document it here.
3. Implement parser/CSS/expr/backends **or** check in expected diagnostics.
4. Parity tests require AOT and runtime to agree on in-subset fixtures.

## SemVer summary

| Bump      | When                                                    |
| --------- | ------------------------------------------------------- |
| **PATCH** | Bug fixes, clearer diagnostics, no grammar change       |
| **MINOR** | New supported syntax covered by fixtures and tests      |
| **MAJOR** | Breaking removal or semantic change to supported syntax |
