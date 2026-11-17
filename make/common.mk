# Shared variables. Included from the root Makefile; do not run this file alone.

SHELL := /bin/bash
ROOT := $(abspath .)
CARGO ?= cargo
CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic -D clippy::nursery
APP_VERSION := $(shell awk '/^\[workspace.package\]/{p=1;next} p&&/^version = /{gsub(/"/,"",$$3); print $$3; exit}' Cargo.toml)
## Independent consumer crate (facade-only, SSR + hydration) used by tests and the version helper.
CONSUMER_FIXTURE := $(ROOT)/crates/rangular-build/tests/fixtures/document-consumer
TAG ?= dev
