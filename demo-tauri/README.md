# Desktop demo (Tauri)

Thin [Tauri](https://v2.tauri.app/) 2 shell around the same Trunk / Leptos CSR SPA
in [`../demo-leptos/`](../demo-leptos/). No `invoke` commands: the webview loads
the wasm demo only.

Stack: **Tauri → Leptos → rangular**.

## Release installers

Pre-built bundles ship on **[GitHub Releases](https://github.com/Interchouette-ITC/rangular/releases)**:

- Linux: `.deb` and AppImage
- Windows: NSIS `.exe`
- macOS: not in v1

Download from the latest release assets, then install or run the AppImage / setup
exe. The browser demo stays at
[https://rangular.interchouette.net](https://rangular.interchouette.net).

## Run locally

```bash
make demo-tauri
```

Opens a native window pointed at `http://127.0.0.1:4180/` (Trunk). If that port
is already serving the Leptos demo, the shell reuses it.

Local release build:

```bash
make demo-tauri-build
```

Artifacts under `demo-tauri/src/target/release/bundle/`.

## Linux dependencies

Build/run needs WebKitGTK 4.1 and GTK development packages (distro names vary),
for example on Debian/Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev
```

Also: Rust stable or nightly, [`trunk`](https://trunkrs.dev/) 0.21.x, and
`cargo install tauri-cli --version '^2'`.

## Icons

Generated from `docs/assets/logo-256.png` via `cargo tauri icon`.
