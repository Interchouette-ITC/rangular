# [**rangular**](https://github.com/Interchouette-ITC/rangular) docs

<p align="center">
  <img src="assets/logo-128.png" alt="rangular mark" width="96" height="96" />
</p>

Start at the [root README](../README.md) for a friendly overview, including how
to depend on git `dev` until crates.io. This folder holds the contract and
contributor habits.

| Doc | What it is |
| --- | --- |
| [`SPEC.md`](SPEC.md) | Language contract for **0.1.x** (what is in / out of scope) |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Fixtures, diagnostics, PR habits |

## Scope in one glance

- **In:** browser DOM via Leptos CSR / wasm, external `.html` + `.scss`, AOT by
  default, runtime for tests.
- **Out (v0.1):** full Angular; i18n / NgModule / DI; **native desktop GUI
  toolkits**.

A [Tauri](https://v2.tauri.app/) (or similar) **webview** still counts as the
browser path: you ship the same wasm UI inside a desktop shell. That is not a
separate **[rangular](https://github.com/Interchouette-ITC/rangular)** backend.

## Related projects

External references (not dependencies):

- [Angust](https://github.com/TudorOrban/Angust) - proposed Angular-style Rust
  GUI on native / desktop widgets (outside a webview)
- [Angular Rust](https://github.com/angular-rust) - Angular-inspired Rust UX
  ecosystem

**[rangular](https://github.com/Interchouette-ITC/rangular)** stays Leptos CSR /
wasm. For a desktop window, put that same UI in a [Tauri](https://v2.tauri.app/)
(or similar) webview; see the root README.
