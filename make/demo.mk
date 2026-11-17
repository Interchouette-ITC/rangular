# Demo Leptos (CSR / wasm, Trunk) and demo Tauri (desktop webview).

.PHONY: demo demo-leptos demo-build demo-leptos-build demo-check demo-leptos-check \
	demo-desktop demo-tauri demo-desktop-build demo-tauri-build

DEMO_LEPTOS := $(ROOT)/demo-leptos
DEMO_TAURI := $(ROOT)/demo-tauri/src
TRUNK := env -u NO_COLOR $(HOME)/.cargo/bin/trunk
DEMO_PORT ?= 4180
DEMO_ADDR ?= 127.0.0.1

demo demo-leptos:
	@if ss -tlnp 2>/dev/null | grep -q ':$(DEMO_PORT) '; then \
		echo "Port $(DEMO_PORT) already in use - reuse that server or set DEMO_PORT"; \
		exit 1; \
	fi
	cd $(DEMO_LEPTOS) && $(TRUNK) serve --release --port $(DEMO_PORT) --address $(DEMO_ADDR)

demo-build demo-leptos-build:
	cd $(DEMO_LEPTOS) && $(TRUNK) build --release

demo-check demo-leptos-check: format-check
	cd $(DEMO_LEPTOS) && $(CARGO) clippy --target wasm32-unknown-unknown --all-targets -- $(CLIPPY_FLAGS)

demo-desktop demo-tauri:
	@if pgrep -f 'rangular-demo-tauri' >/dev/null 2>&1; then \
		echo "rangular-demo-tauri already running - reuse that window"; \
		exit 0; \
	fi
	@if ss -tlnp 2>/dev/null | grep -q ':$(DEMO_PORT) '; then \
		echo "Port $(DEMO_PORT) in use - Tauri will attach without starting Trunk"; \
		cd $(DEMO_TAURI) && $(CARGO) tauri dev --config '{"build":{"beforeDevCommand":""}}'; \
	else \
		cd $(DEMO_TAURI) && $(CARGO) tauri dev; \
	fi

demo-desktop-build demo-tauri-build:
	cd $(DEMO_TAURI) && $(CARGO) tauri build
