# rangular developer targets

SHELL := /bin/bash
ROOT := $(abspath .)
DEMO_LEPTOS := $(ROOT)/demo-leptos
DEMO_TAURI := $(ROOT)/demo-tauri/src
CARGO ?= cargo
TRUNK := env -u NO_COLOR $(HOME)/.cargo/bin/trunk
CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery
DEMO_PORT ?= 4180
DEMO_ADDR ?= 127.0.0.1
DOCKER_PORT ?= 8080
DOCKER_BUILDKIT ?= 1
APP_VERSION := $(shell awk '/^\[workspace.package\]/{p=1;next} p&&/^version = /{gsub(/"/,"",$$3); print $$3; exit}' Cargo.toml)
HUB_IMAGE := interchouette/rangular-demo
GHCR_PERSONAL_IMAGE := ghcr.io/groussac/rangular-demo
GHCR_WORKER_IMAGE := ghcr.io/interchouette/rangular-demo
GHCR_ORG_IMAGE := ghcr.io/interchouette-itc/rangular-demo
DOCKERFILE ?= docker/Dockerfile
TAG ?= dev

.DEFAULT_GOAL := help

.PHONY: help check test lint format format-check clean ci no-panic coverage \
	demo demo-leptos demo-build demo-leptos-build demo-check demo-leptos-check \
	demo-desktop demo-tauri demo-desktop-build demo-tauri-build \
	docker-build docker-build-dev docker-run \
	docker-push-dev-hub docker-push-dev-ghcr-personal docker-push-dev-ghcr-itc \
	docker-push-release-hub docker-push-release-ghcr-personal docker-push-release-ghcr-itc \
	version-show version-bump-patch version-bump-minor version-bump-major version-set \
	audit deny

help:
	@echo "rangular targets"
	@echo ""
	@echo "  make check         cargo check --workspace"
	@echo "  make test          cargo test --workspace"
	@echo "  make no-panic      garbage fixtures (parser/aot/runtime)"
	@echo "  make lint          fmt check + clippy (workspace)"
	@echo "  make ci            lint + test + no-panic"
	@echo "  make coverage      cargo llvm-cov → coverage/lcov.info"
	@echo "  make audit         cargo audit"
	@echo "  make deny          cargo deny check"
	@echo "  make format        cargo fmt"
	@echo "  make clean         cargo clean"
	@echo ""
	@echo "Demo Leptos (CSR / wasm):"
	@echo "  make demo / demo-leptos          Trunk serve → http://$(DEMO_ADDR):$(DEMO_PORT)/"
	@echo "  make demo-build / demo-leptos-build"
	@echo "  make demo-check / demo-leptos-check"
	@echo ""
	@echo "Demo Tauri:"
	@echo "  make demo-tauri / demo-desktop   Tauri window (reuses :$(DEMO_PORT) if up)"
	@echo "  make demo-tauri-build / demo-desktop-build"
	@echo ""
	@echo "Docker (browser SPA for Render):"
	@echo "  make docker-build      Build $(HUB_IMAGE):$(APP_VERSION) + :latest"
	@echo "  make docker-build-dev  Tag :dev + :latest (Hub + GHCR names)"
	@echo "  make docker-run        Run image on :$(DOCKER_PORT)"
	@echo ""
	@echo "Version:"
	@echo "  make version-show"
	@echo "  make version-bump-patch|minor|major"
	@echo "  make version-set VERSION=x.y.z"
	@echo "  Release: wait until CI + tip Docker are green, then create GitHub Release tag v\$$(APP_VERSION)"
	@echo ""
	@echo "Overrides: DEMO_PORT=$(DEMO_PORT) DEMO_ADDR=$(DEMO_ADDR) DOCKER_PORT=$(DOCKER_PORT)"

check:
	cd $(ROOT) && $(CARGO) check --workspace

test:
	cd $(ROOT) && $(CARGO) test --workspace

no-panic:
	cd $(ROOT) && $(CARGO) test -p rangular-parser garbage_input_never_panics -- --exact
	cd $(ROOT) && $(CARGO) test -p rangular-aot garbage_input_returns_issues_not_empty_code -- --exact
	cd $(ROOT) && $(CARGO) test -p rangular-runtime garbage_never_panics -- --exact

format:
	cd $(ROOT) && $(CARGO) fmt

format-check:
	cd $(ROOT) && $(CARGO) fmt --check

lint: format-check
	cd $(ROOT) && $(CARGO) clippy --workspace --all-targets -- $(CLIPPY_FLAGS)

## Requires `cargo install cargo-audit`.
audit:
	cd $(ROOT) && $(CARGO) audit

## Requires `cargo install cargo-deny`.
deny:
	cd $(ROOT) && $(CARGO) deny check

## Requires `cargo install cargo-llvm-cov`. Writes `coverage/lcov.info`.
coverage:
	cd $(ROOT) && mkdir -p coverage && RUSTUP_TOOLCHAIN=stable $(CARGO) llvm-cov --workspace --lcov --output-path coverage/lcov.info

ci: lint test no-panic

clean:
	cd $(ROOT) && $(CARGO) clean

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

docker-build:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build --pull --network=host \
		-f $(DOCKERFILE) -t $(HUB_IMAGE):$(APP_VERSION) $(ROOT)
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(HUB_IMAGE):latest

docker-build-dev:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build --pull --network=host \
		-f $(DOCKERFILE) -t $(HUB_IMAGE):dev $(ROOT)
	docker tag $(HUB_IMAGE):dev $(HUB_IMAGE):latest
	docker tag $(HUB_IMAGE):dev $(GHCR_PERSONAL_IMAGE):dev
	docker tag $(HUB_IMAGE):dev $(GHCR_PERSONAL_IMAGE):latest
	docker tag $(HUB_IMAGE):dev $(GHCR_WORKER_IMAGE):dev
	docker tag $(HUB_IMAGE):dev $(GHCR_WORKER_IMAGE):latest
	docker tag $(HUB_IMAGE):dev $(GHCR_ORG_IMAGE):dev
	docker tag $(HUB_IMAGE):dev $(GHCR_ORG_IMAGE):latest

docker-run: docker-build-dev
	@if ss -tlnp 2>/dev/null | grep -q ':$(DOCKER_PORT) '; then \
		echo "Port $(DOCKER_PORT) already in use"; \
		exit 1; \
	fi
	docker run --rm -p $(DOCKER_PORT):8080 -e PORT=8080 $(HUB_IMAGE):latest

docker-push-dev-hub:
	docker push $(HUB_IMAGE):dev
	docker push $(HUB_IMAGE):latest

docker-push-dev-ghcr-personal:
	docker push $(GHCR_PERSONAL_IMAGE):dev
	docker push $(GHCR_PERSONAL_IMAGE):latest

docker-push-dev-ghcr-itc:
	docker push $(GHCR_WORKER_IMAGE):dev
	docker push $(GHCR_WORKER_IMAGE):latest
	docker push $(GHCR_ORG_IMAGE):dev
	docker push $(GHCR_ORG_IMAGE):latest

docker-push-release-hub:
	docker push $(HUB_IMAGE):$(APP_VERSION)
	docker push $(HUB_IMAGE):latest

docker-push-release-ghcr-personal:
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_PERSONAL_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_PERSONAL_IMAGE):latest
	docker push $(GHCR_PERSONAL_IMAGE):$(APP_VERSION)
	docker push $(GHCR_PERSONAL_IMAGE):latest

docker-push-release-ghcr-itc:
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_WORKER_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_WORKER_IMAGE):latest
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_ORG_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_ORG_IMAGE):latest
	docker push $(GHCR_WORKER_IMAGE):$(APP_VERSION)
	docker push $(GHCR_WORKER_IMAGE):latest
	docker push $(GHCR_ORG_IMAGE):$(APP_VERSION)
	docker push $(GHCR_ORG_IMAGE):latest

# Version helpers (workspace + demos + tauri.conf); release via GitHub Release tag v$$(APP_VERSION)
version-show:
	@echo "Current version: $(APP_VERSION)"; \
	echo "demo-leptos:     $$(awk '/^version = /{gsub(/"/,"",$$3); print $$3; exit}' demo-leptos/Cargo.toml)"; \
	echo "demo-tauri:      $$(awk '/^version = /{gsub(/"/,"",$$3); print $$3; exit}' demo-tauri/src/Cargo.toml)"; \
	echo "tauri.conf.json: $$(python3 -c "import json; print(json.load(open('demo-tauri/src/tauri.conf.json'))['version'])")"; \
	echo ""; \
	echo "Suggested GitHub Release tag:"; \
	echo "  v$(APP_VERSION)"; \
	echo ""; \
	echo "When creating a GitHub Release, use the Tag field (not only the title)."

define version-apply
	@current="$(APP_VERSION)"; \
	new="$(1)"; \
	if [ -z "$$new" ]; then echo "empty version"; exit 1; fi; \
	sed -i "s/version = \"$$current\"/version = \"$$new\"/g" Cargo.toml; \
	sed -i "s/^version = \"$$current\"/version = \"$$new\"/" demo-leptos/Cargo.toml; \
	sed -i "s/^version = \"$$current\"/version = \"$$new\"/" demo-tauri/src/Cargo.toml; \
	python3 -c "import json; p='demo-tauri/src/tauri.conf.json'; c=json.load(open(p)); c['version']='$$new'; json.dump(c, open(p,'w'), indent=2); open(p,'a').write('\n')"; \
	$(CARGO) metadata --format-version 1 --no-deps >/dev/null; \
	$(CARGO) metadata --manifest-path demo-leptos/Cargo.toml --format-version 1 --no-deps >/dev/null; \
	$(CARGO) metadata --manifest-path demo-tauri/src/Cargo.toml --format-version 1 --no-deps >/dev/null; \
	echo "Version $$current → $$new (workspace + demos + tauri.conf)"
endef

version-bump-patch:
	$(call version-apply,$(shell echo "$(APP_VERSION)" | awk -F. '{print $$1"."$$2"."($$3+1)}'))

version-bump-minor:
	$(call version-apply,$(shell echo "$(APP_VERSION)" | awk -F. '{print $$1"."($$2+1)".0"}'))

version-bump-major:
	$(call version-apply,$(shell echo "$(APP_VERSION)" | awk -F. '{print ($$1+1)".0.0"}'))

version-set:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make version-set VERSION=x.y.z"; \
		exit 1; \
	fi
	$(call version-apply,$(VERSION))
