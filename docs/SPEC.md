# [**rangular**](https://github.com/Interchouette-ITC/rangular) specification v0.1

This document is the language contract for
**[rangular](https://github.com/Interchouette-ITC/rangular)** **0.1.x**. Patch
releases clarify diagnostics and fix bugs without breaking supported syntax.
Minor releases may add constructs when fixtures and tests cover them. Major
releases may remove or rename supported constructs only with a migration note.

## Scope

| In scope                                       | Out of scope (v0.1)                                      |
| ---------------------------------------------- | -------------------------------------------------------- |
| Web DOM via Leptos CSR                         | Full Angular framework                                   |
| External `.html` templates                     | Markup inside Rust controllers                           |
| Component `.scss` (compiled then encapsulated) | Pipes, two-way banana `[(…)]`, named slots / `ng-template` |
| AOT (production default)                       | i18n, NgModule, Angular DI                               |
| Runtime interpreter (tests/tooling)            | Claiming runtime as the production path                  |
| Fixture-driven growth                          | Native desktop GUI toolkits                              |
| Default `<ng-content>`, `EventPayload`, Input/Output | Silent ignore of unknown syntax                    |

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

Handler names resolve through the host `call` API. `$event` is a typed
[`EventPayload`](../crates/rangular-host/src/event.rs) (`Click`, `Input { value }`,
`Error`, or `Custom`) exposed as `Value::Event` (see fixture
[`tests/fixtures/html/event-payload.html`](../tests/fixtures/html/event-payload.html)).

### Content projection

| Surface syntax              | Meaning                                      |
| --------------------------- | -------------------------------------------- |
| `<ng-content></ng-content>` | Default projection slot (layout shells)      |

Fixture: [`tests/fixtures/components/layout-shell/`](../tests/fixtures/components/layout-shell/).
AOT view factories that contain `<ng-content>` take a Leptos `Children` argument
and insert `{children()}` at the slot. Runtime uses `interpret_with_slot` /
`render_with_slot` to inject projected `VNode`s.

**Not in v0.1:** `<ng-content select="…">`, `<ng-template #ref>`, or
`ngTemplateOutlet`.

### Component Input / Output (growth)

Parent ↔ child communication uses `[input]` and `(output)` on registered tags.
Teaching fixtures:

- [`tests/fixtures/components/io-child/`](../tests/fixtures/components/io-child/)
- [`tests/fixtures/html/io-parent.html`](../tests/fixtures/html/io-parent.html)

`Registry` stores input / output names per tag (`app-io-child`, `app-chrome-header`).
[`classify_bindings`](../crates/rangular-parser/src/component_io.rs) rewrites matching
`[prop]` / `(event)` into `Input` / `Output` attrs. AOT emits hyphenated tags as
`div[data-rangular-component=…]` host shells with `data-input-*` /
`data-output-*` attrs; runtime snapshots use `input:` / `output:` labels.
Full view-fn composition remains a follow-up.

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
`togglePalette()` or `onColorInput($event)`.

**Not in v0.1:** pipes (`{{ x | number }}`), ternary, nullish coalescing,
assignment, two-way banana `[(…)]`, arbitrary method chains beyond what the
host exposes.

## Reference template: color-field

Canonical example from
[`tests/fixtures/components/color-field/`](../tests/fixtures/components/color-field/)
(also `chrome-header`, `asset-icon` in the same folder):

```html
<div class="color-field">
  <span class="color-field__label">{{ label }}</span>
  <input
    [id]="inputId"
    type="color"
    class="color-field__color"
    [value]="value"
    (input)="onColorInput($event)"
  />
  <button
    type="button"
    class="color-field__toggle"
    [class.color-field__toggle--open]="paletteOpen"
    [attr.aria-expanded]="paletteOpen"
    (click)="togglePalette()"
  >
    Palette
  </button>
</div>
```

The host exposes bindings such as `label`, `value`, `paletteOpen`, and the
handler methods. Class names that are only **global** app utilities (for example
`.btn`) pass through unscoped; component classes such as `.color-field` are
scoped by encapsulation when you use that path.

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

| Operation          | Purpose                                          |
| ------------------ | ------------------------------------------------ |
| `get(path)`        | Read a binding value (`label`, `paletteOpen`, …) |
| `set(path, value)` | Two-way / input side effects when needed         |
| `call(name, args)` | Invoke controller handlers                       |
| Event bridge       | Map `(click)` etc. to host callables             |

## Inject / registry

Typed services use `Registry::provide` / `Registry::inject` (TypeId map).
Component custom tags resolve through the same registry. Apps call
`register_tag` for their own components. `Registry::with_example_panels()`
registers tags that match the fixture corpus:

| Tag                 | Component    |
| ------------------- | ------------ |
| `app-item-list`     | ItemList     |
| `app-color-field`   | ColorField   |
| `app-chrome-header` | ChromeHeader |
| `app-asset-icon`    | AssetIcon    |

At the Leptos app edge, apps map this to `provide_context` / `use_context` as
needed.

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
- `RANG401` empty template (AOT and runtime)
- `RANG501`–`RANG599` other runtime interpret errors

AOT may promote selected warnings to compile errors when configured; default
builds fail on `RANG001` and `RANG201` for in-scope fixtures.

Production builds use **AOT**. The **runtime** interpreter shares the same AST,
Host API, and structural binding IR for parity tests and tooling; it is not the
production path. Parity requires AOT and runtime to agree on in-subset fixtures
(`ok` outcome, shared IR snapshot, and host-evaluated runtime goldens where a
fixture host exists).

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
5. **Gate:** `rangular-parser` integration test `required_fixtures_exist` fails
   if a planned construct path is missing; SPEC must name the fixture paths for
   `EventPayload`, `ng-content`, `layout-shell`, and `io-child`.

## SemVer summary

| Bump      | When                                                    |
| --------- | ------------------------------------------------------- |
| **PATCH** | Bug fixes, clearer diagnostics, no grammar change       |
| **MINOR** | New supported syntax covered by fixtures and tests      |
| **MAJOR** | Breaking removal or semantic change to supported syntax |
