# Desktop demo (Tauri)

Thin [Tauri](https://v2.tauri.app/) 2 shell around the same Trunk / Leptos CSR SPA
in [`../demo/`](../demo/). No `invoke` commands: the webview loads the wasm
demo only.

Stack: **Tauri → Leptos → rangular**.

## Run

```bash
make demo-desktop
```

Opens a native window pointed at `http://127.0.0.1:4180/` (Trunk). If that port
is already serving the browser demo, the shell reuses it.

Release bundles (Linux `deb` / AppImage, Windows NSIS):

```bash
make demo-desktop-build
```

Artifacts under `demo-desktop/src-tauri/target/release/bundle/`.

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
