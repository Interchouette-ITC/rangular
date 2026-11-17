# Releasing rangular

All nine workspace packages share one version (`[workspace.package]` in the
root `Cargo.toml`) and are released together: GitHub Release, Docker image,
Tauri installers, and crates.io.

## Packages

| Package            | Audience                                   |
| ------------------ | ------------------------------------------ |
| `rangular`         | Applications (`[dependencies]`)            |
| `rangular-build`   | Applications (`[build-dependencies]`)      |
| other seven        | Implementation crates behind the facade    |

The implementation crates are published only because Cargo requires every
registry dependency of `rangular` / `rangular-build` to be on crates.io.

## Publication order

`cargo publish` uploads one package at a time, and a package can only be
published once its versioned dependencies are on the registry index:

1. `rangular-host`
2. `rangular-expr` (host)
3. `rangular-parser` (expr)
4. `rangular-css`
5. `rangular-macros`
6. `rangular-aot` (expr, host, parser)
7. `rangular-runtime` (expr, host, parser; dev-dependency aot)
8. `rangular-build` (aot, css)
9. `rangular` (aot, css, expr, host, macros, parser, runtime)

`rangular-host` has a path-only dev-dependency on `rangular-expr`; Cargo
removes it from the published manifest, which breaks the host ↔ expr cycle.
[`make/publish/publish-crates.sh`](../make/publish/publish-crates.sh) holds this list
and fails if it drifts from the workspace members.

## Checklist

1. `make version-set VERSION=x.y.z` (workspace, demos, `tauri.conf.json`,
   lockfiles including the consumer fixture, and the `rangular = "x.y.z"` /
   `rangular-build = "x.y.z"` installation snippets listed in `VERSIONED_DOCS`
   in the `Makefile`).
2. Move `[Unreleased]` entries into a new section of
   [`CHANGELOG.md`](CHANGELOG.md).
3. Validate locally:

   ```bash
   make lint test no-panic doc
   make test-hydration   # CHROME_BIN=... if Chrome and chromedriver differ
   make publish-check    # PUBLISH_CHECK_ARGS=--allow-dirty before committing
   ```

   `make publish-check` runs `cargo package --workspace --locked`, which
   builds every package from its `.crate` file against a temporary local
   registry, so dependents are verified before their dependencies are on
   crates.io.
4. Merge, wait for CI and the dev Docker image, then create the GitHub
   Release with tag `vx.y.z` (published, not draft or pre-release). The
   release workflow reruns CI on the tag anyway, so a red check blocks
   publication even if it was missed here.

## What the release workflow does

[`release.yml`](../.github/workflows/release.yml) validates the tag against
the workspace and demo versions, then runs the whole
[`ci.yml`](../.github/workflows/ci.yml) workflow on the tagged commit (its `ci`
job: lint, tests, MSRV 1.100, no-panic, demos, hydration, rustdoc,
supply-chain, coverage). The `crates`, `docker` and `desktop` jobs need both
`validate` and `ci`, so nothing is published unless every check passes. Then
the `crates` job, on Rust 1.100:

1. runs `make publish-check` on the tag;
2. runs `make/publish/publish-crates.sh publish` with the `CARGO_REGISTRY_TOKEN`
   repository secret, exposed only to that step. The script:
   - fails immediately if the token is empty;
   - does not re-upload versions that already exist on crates.io, but still
     waits for each of them in the sparse index, so a rerun after a partial
     failure (including an index timeout) resumes where it stopped;
   - publishes in the order above with `cargo publish --locked` (full
     verification, never `--no-verify`);
   - waits for each version to appear in the sparse index before the next
     package (bounded: `INDEX_ATTEMPTS` × `INDEX_INTERVAL` seconds, default
     10 minutes) and fails on timeout.

The job is serialized (`concurrency: crates-io-publish`) and skipped for
draft and pre-release releases. The workflow runs on `release: created`, so a
release saved as a draft and published later does not trigger it; publish
from a clean checkout of the tag with `make/publish/publish-crates.sh publish`
instead. That manual path bypasses the CI gate: confirm CI is green on the
tagged commit first. A crates.io version cannot be replaced; fix
forward with a new patch release, or `cargo yank` a broken version.

## First publication (1.1.0)

The nine package names do not exist on crates.io before 1.1.0, so the first
run creates them:

- The `CARGO_REGISTRY_TOKEN` token needs the **publish-new** and
  **publish-update** scopes (restrict it to `rangular*` crates if you scope
  by crate name). Its owner becomes the initial owner of every crate.
- After the run, add co-owners to each crate, for example
  `cargo owner --add github:Interchouette-ITC:<team> <crate>`.
- If you prefer to publish the first release by hand, run
  `CARGO_REGISTRY_TOKEN=... make/publish/publish-crates.sh publish` from a clean
  checkout of the tag; the workflow then skips the existing versions.

## Later releases

Only **publish-update** is required once the crates exist. Bump the version,
follow the checklist, and create the GitHub Release; the workflow publishes
all nine packages at the new version.
