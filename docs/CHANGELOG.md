# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
