# rangular developer targets

SHELL := /bin/bash
ROOT := $(abspath .)
CARGO ?= cargo
CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery

.DEFAULT_GOAL := help

.PHONY: help check test lint format format-check clean ci no-panic

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
