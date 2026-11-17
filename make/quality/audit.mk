# Dependency and supply-chain checks.

.PHONY: audit deny machete outdated geiger

## Requires `cargo install cargo-audit`.
audit:
	cd $(ROOT) && $(CARGO) audit

## Requires `cargo install cargo-deny`.
deny:
	cd $(ROOT) && $(CARGO) deny check

## Unused workspace dependencies. Requires `cargo install cargo-machete`.
machete:
	cd $(ROOT) && $(CARGO) machete

## Outdated crates report. Requires `cargo install cargo-outdated`.
outdated:
	cd $(ROOT) && $(CARGO) outdated --workspace

## Unsafe Rust surface in deps + workspace. Requires `cargo install cargo-geiger`.
geiger:
	cd $(ROOT) && $(CARGO) geiger --workspace
