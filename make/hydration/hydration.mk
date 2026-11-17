# Native SSR render of the consumer fixture, hydrated in headless Chrome.
# Heavy body: make/hydration/test-hydration.sh. Requires the wasm32 target,
# wasm-bindgen-test-runner matching the fixture lockfile, and chromedriver.

.PHONY: test-hydration

HYDRATION_OUT := $(ROOT)/target/hydration
CHROMEDRIVER ?= chromedriver
## Optional Chrome/Chromium binary matching the chromedriver major version.
CHROME_BIN ?=

test-hydration:
	CARGO="$(CARGO)" CONSUMER_FIXTURE="$(CONSUMER_FIXTURE)" HYDRATION_OUT="$(HYDRATION_OUT)" \
		CHROMEDRIVER="$(CHROMEDRIVER)" CHROME_BIN="$(CHROME_BIN)" \
		$(ROOT)/make/hydration/test-hydration.sh
