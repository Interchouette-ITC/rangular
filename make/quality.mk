# Build, test, and lint the workspace.

.PHONY: check test no-panic format format-check lint clean ci

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

clean:
	cd $(ROOT) && $(CARGO) clean

ci: lint test no-panic doc audit deny

include make/quality/audit.mk
include make/quality/coverage.mk
include make/quality/fuzz.mk
include make/quality/doc.mk
