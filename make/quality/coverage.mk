# Coverage reports.

.PHONY: coverage coverage-summary coverage-html tarpaulin

## Requires `cargo install cargo-llvm-cov`. Writes `coverage/lcov.info` (CI / Codecov).
## demo hosts are workspace-excluded; regex keeps accidental paths out of lcov.
COVERAGE_IGNORE := demo-leptos/|demo-tauri/

coverage:
	cd $(ROOT) && mkdir -p coverage && RUSTUP_TOOLCHAIN=stable $(CARGO) llvm-cov --workspace --locked --lcov \
		--ignore-filename-regex '$(COVERAGE_IGNORE)' \
		--output-path coverage/lcov.info

## Terminal summary only (fast local check).
coverage-summary:
	cd $(ROOT) && RUSTUP_TOOLCHAIN=stable $(CARGO) llvm-cov --workspace --locked --summary-only \
		--ignore-filename-regex '$(COVERAGE_IGNORE)'

## HTML report → `coverage/html/`.
coverage-html:
	cd $(ROOT) && mkdir -p coverage && RUSTUP_TOOLCHAIN=stable $(CARGO) llvm-cov --workspace --locked --html \
		--ignore-filename-regex '$(COVERAGE_IGNORE)' \
		--output-dir coverage/html

## Alternate local coverage via tarpaulin (not used by CI; Codecov stays on llvm-cov).
## Requires `cargo install cargo-tarpaulin`.
tarpaulin:
	cd $(ROOT) && mkdir -p coverage/tarpaulin && $(CARGO) tarpaulin --workspace --locked \
		--exclude-files 'demo-leptos/*' 'demo-tauri/*' \
		--out Html --out Xml --output-dir coverage/tarpaulin
