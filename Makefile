# rangular developer targets

SHELL := /bin/bash
ROOT := $(abspath .)
DEMO := $(ROOT)/demo
CARGO ?= cargo
TRUNK := env -u NO_COLOR $(HOME)/.cargo/bin/trunk
CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery
DEMO_PORT ?= 4180
DEMO_ADDR ?= 127.0.0.1
DOCKER_PORT ?= 8080
DOCKER_BUILDKIT ?= 1
APP_VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
HUB_IMAGE := interchouette/rangular-demo
GHCR_PERSONAL_IMAGE := ghcr.io/groussac/rangular-demo
GHCR_WORKER_IMAGE := ghcr.io/interchouette/rangular-demo
GHCR_ORG_IMAGE := ghcr.io/interchouette-itc/rangular-demo
DOCKERFILE ?= docker/Dockerfile
TAG ?= dev

.DEFAULT_GOAL := help

.PHONY: help check test lint format format-check clean ci no-panic \
	demo demo-build demo-check \
	docker-build docker-build-dev docker-run \
	docker-push-dev-hub docker-push-dev-ghcr-personal docker-push-dev-ghcr-itc

help:
	@echo "rangular targets"
	@echo ""
	@echo "  make check         cargo check --workspace"
	@echo "  make test          cargo test --workspace"
	@echo "  make no-panic      garbage fixtures (parser/aot/runtime)"
	@echo "  make lint          fmt check + clippy (workspace)"
	@echo "  make ci            lint + test + no-panic"
	@echo "  make format        cargo fmt"
	@echo "  make clean         cargo clean"
	@echo ""
	@echo "Demo (Leptos CSR / wasm):"
	@echo "  make demo          Trunk serve → http://$(DEMO_ADDR):$(DEMO_PORT)/"
	@echo "  make demo-build    Trunk release dist in demo/"
	@echo "  make demo-check    fmt + clippy on demo wasm target"
	@echo ""
	@echo "Docker (browser SPA for Render):"
	@echo "  make docker-build      Build $(HUB_IMAGE):$(APP_VERSION)"
	@echo "  make docker-build-dev  Tag :dev + :latest (Hub + GHCR names)"
	@echo "  make docker-run        Run image on :$(DOCKER_PORT)"
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

ci: lint test no-panic

clean:
	cd $(ROOT) && $(CARGO) clean

demo:
	@if ss -tlnp 2>/dev/null | grep -q ':$(DEMO_PORT) '; then \
		echo "Port $(DEMO_PORT) already in use - reuse that server or set DEMO_PORT"; \
		exit 1; \
	fi
	cd $(DEMO) && $(TRUNK) serve --release --port $(DEMO_PORT) --address $(DEMO_ADDR)

demo-build:
	cd $(DEMO) && $(TRUNK) build --release

demo-check: format-check
	cd $(DEMO) && $(CARGO) clippy --target wasm32-unknown-unknown --all-targets -- $(CLIPPY_FLAGS)

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
