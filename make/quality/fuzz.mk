# Fuzzing.

.PHONY: fuzz fuzz-build

## Default fuzz target: rangular_parser::parse (Angular-subset HTML).
## Requires nightly + `cargo install cargo-fuzz`. Override: FUZZ_TARGET=… FUZZ_TIME=…
FUZZ_TARGET ?= parse-html
FUZZ_TIME ?= 10

fuzz-build:
	cd $(ROOT) && cargo +nightly fuzz build

fuzz:
	cd $(ROOT) && cargo +nightly fuzz run $(FUZZ_TARGET) -- -max_total_time=$(FUZZ_TIME)
