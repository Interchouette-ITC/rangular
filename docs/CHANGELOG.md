# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.6] - 2026-11-17

First release prepared for publication on [crates.io](https://crates.io/).

### Changed

- **Breaking for generated code:** AOT output now references Rangular only
  through the `rangular` facade (`rangular::host`, `rangular::aot`,
  `rangular::expr`) instead of `rangular_host`, `rangular_aot`, and
  `rangular_expr`. Consumers depend on `rangular` (plus `rangular-build` as a
  build dependency) rather than the implementation crates, and should import
  `rangular::aot::HostCell` / `rangular::host::…`. Templates that bind DOM events
  or two-way form values still need direct `web-sys` (and `wasm-bindgen`)
  dependencies.
- Minimum supported Rust version is now **1.100** for the workspace and both
  demos; the Docker builder image moves to `rust:1.100.0-bookworm`.
- `demo-leptos` depends on the `rangular` facade only; `demo-tauri` and
  `tauri.conf.json` are aligned to 1.0.6.
- All nine packages are publishable: `publish = false` removed, and each has a
  README, the Apache-2.0 `LICENSE` text (`license-file.workspace = true`),
  keywords, and categories. `rangular-host`'s dev-dependency on
  `rangular-expr` is path-only so the published dependency graph has no cycle.
- `make version-set` / `version-bump-*` also refresh the consumer fixture
  lockfile and the `rangular` / `rangular-build` installation snippets in the
  READMEs.
- The `Makefile` is a thin index that includes `make/*.mk`; long recipes live
  in companion scripts (`make/hydration/test-hydration.sh`,
  `make/version/version-apply.sh`, `make/publish/publish-crates.sh`). Target
  names and overrides are unchanged.

### Added

- Native SSR and SSR-to-browser hydration tests: a consumer crate renders a
  counter component on the server, and `make test-hydration` (CI job
  `hydration`) hydrates that exact HTML in headless Chrome, checking that
  server DOM nodes are reused and that events and reactive updates work. A
  negative test ensures a fresh client render is detected.
- A test asserting generated code contains no implementation-crate paths.
- crates.io publication: `make/publish/publish-crates.sh` (dependency-ordered,
  skips already-published versions, waits for the index), `make publish-check`,
  and a `crates` job in the release workflow using the `CARGO_REGISTRY_TOKEN`
  secret. See [`RELEASING.md`](RELEASING.md).
- `docs/RELEASING.md` and a `rangular-build` README.

### Fixed

- The `document_consumer` test failed under `cargo test --locked` because the
  consumer fixture's `Cargo.lock` was stale after the version bump; it is
  refreshed and kept in sync by the version helper.
- The version helper ran `cargo metadata --no-deps`, which left the workspace
  and demo lockfiles at the previous version and broke later `--locked`
  commands; it now refreshes every lockfile and fails if a refresh fails.

### Documentation

- The guide leads with `rangular` + `rangular-build`, documents the facade
  modules, the package layout and publication model, and keeps manual AOT
  compilation as an advanced `build.rs` option.
- CSR, SSR, and hydration are stated consistently as supported targets across
  `README.md`, `OVERVIEW.md`, and `SPEC.md`, with their test coverage.
- The `rangular` crate README contains a complete working example.

## [1.0.5] - 2026-11-15

### Added

- AOT head component adapters for title, metadata, and link integrations, with
  stable `head-key` registry keys. ([#97](https://github.com/Interchouette-ITC/rangular/pull/97))
- `rangular-build` document templates: `Config::document`, `Config::head`,
  `Config::body`, and optional `Config::head_adapter` compile application-level
  templates through the existing AOT pipeline into `<file-stem>_view.rs`, with
  sibling SCSS for document and body templates. Components become optional when
  a document template is registered; view-name collisions are build errors.
  `HeadComponentAdapter` is re-exported from `rangular-build`.
- Parser, AOT and runtime preserve a leading `<!DOCTYPE html>` in document
  templates as `Node::Doctype`, `IrNode::Doctype` and `VNode::Doctype`; generated
  views and runtime snapshots emit the canonical declaration. Head templates
  reject it with `RANG403`. ([#98](https://github.com/Interchouette-ITC/rangular/pull/98))

## [1.0.4] - 2026-11-11

### Added

- `rangular-build` Cargo helper for component discovery, HTML-to-Rust view
  compilation, and SCSS compilation; generated CSS paths participate in rebuild
  tracking. ([#88](https://github.com/Interchouette-ITC/rangular/pull/88),
  [#89](https://github.com/Interchouette-ITC/rangular/pull/89))
- Opt-in AOT document-head compiler for title, metadata, links, scripts, and
  safely escaped JSON-LD. ([#95](https://github.com/Interchouette-ITC/rangular/pull/95))

### Fixed

- `[checked]` / `[(checked)]` use boolean evaluation and `change`-event writeback,
  so `false` is preserved when updating checkbox state.
  ([#87](https://github.com/Interchouette-ITC/rangular/pull/87))
- Parse interpolated attribute values as bindings and emit explicit HTML
  attributes correctly, including Open Graph metadata.
  ([#90](https://github.com/Interchouette-ITC/rangular/pull/90),
  [#93](https://github.com/Interchouette-ITC/rangular/pull/93))
- Reject unsupported content in head templates, preserve raw non-JSON-LD script
  bodies, and handle case-insensitive HTML tag names and JSON-LD string content.
  ([#95](https://github.com/Interchouette-ITC/rangular/pull/95))

### Changed

- Update the Docker demo build and exclude unnecessary files from its build
  context. ([#91](https://github.com/Interchouette-ITC/rangular/pull/91),
  [#92](https://github.com/Interchouette-ITC/rangular/pull/92))

## [1.0.3] - 2026-09-16

### Changed

- Edition 2024 workspace migration, housecleaning, and CI/Make coverage tooling
- Strict pipe Results; drop AOT clippy allows; named opaque lifetimes
- Dependabot / JamesIves action bumps and gRoussac review requests on Dependabot PRs

## [1.0.2] - 2026-08-26

### Changed

- Code showcase no longer follows page scroll. Hovering a fixture panel (or
  opening a hash link) selects its sources; leaving the panel does not change
  the rail.

## [1.0.1] - 2026-08-26

### Added

- **Floating code showcase** in the Leptos browser demo: a read-only
  [`kode-leptos`](https://crates.io/crates/kode-leptos) panel that shows the live
  fixture sources (HTML template, SCSS, Rust host) for the panel in view.
- Scroll and hash navigation keep the showcase aligned with the active fixture.
- Per-panel file tabs (compact stem + extension bubbles; middle ellipsis when
  names are long). The `io-parent` fixture exposes parent and child triples.
- Binding pulse on `io-child` mute: highlights the `(muteToggle)` line when the
  showcase is already on that panel.
- Preference for open/collapsed state in `localStorage`. On narrow viewports
  (tablet and phone) the rail starts collapsed and opens as a bottom sheet with
  a dismiss backdrop so the demo stays usable.

### Fixed

- Docker image builder bumped from Rust 1.88 to **1.98** so the published wasm
  no longer panics at init (`RuntimeError: unreachable` on the live demo).
- Narrow layouts: contain the `item-list` table scroll, hide decorative overflow,
  and keep page width stable.

### Changed

- Trunk `wasm-opt` pinned to Binaryen **version_132** (was Trunk’s default
  `version_123`) so CI/Docker match a current optimizer.

## [1.0.0] - 2026-08-25

### Added

- First stable cut of the library and demos (browser SPA, Docker image, desktop
  installers via GitHub Releases).
