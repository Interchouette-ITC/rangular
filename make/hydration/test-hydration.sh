#!/usr/bin/env bash
# Renders the consumer fixture's counter view natively (SSR), then hydrates that
# exact HTML in headless Chrome and lints the hydrate build.
#
# Steps:
#   1. Check wasm-bindgen-test-runner (version = fixture lockfile) and chromedriver.
#   2. Render the counter with the `render_counter` example → $HYDRATION_OUT/counter.html.
#   3. Write webdriver.json (headless; "binary" only when CHROME_BIN is set).
#   4. Run the wasm32 `hydration` test against that HTML.
#   5. Clippy the wasm32 hydrate build.
#
# Operator entry: `make test-hydration` (CHROMEDRIVER=..., CHROME_BIN=..., CARGO=...).
# Env: CARGO, CONSUMER_FIXTURE, HYDRATION_OUT, CHROMEDRIVER, CHROME_BIN.
set -euo pipefail

ROOT="$(cd -- "$(dirname "$0")/../.." && pwd)"
CARGO=${CARGO:-cargo}
CONSUMER_FIXTURE=${CONSUMER_FIXTURE:-$ROOT/crates/rangular-build/tests/fixtures/document-consumer}
HYDRATION_OUT=${HYDRATION_OUT:-$ROOT/target/hydration}
CHROMEDRIVER=${CHROMEDRIVER:-chromedriver}
CHROME_BIN=${CHROME_BIN:-}

command -v wasm-bindgen-test-runner >/dev/null \
  || { echo "missing wasm-bindgen-test-runner (cargo install wasm-bindgen-cli --locked --version <fixture wasm-bindgen>)"; exit 1; }
command -v "$CHROMEDRIVER" >/dev/null \
  || { echo "missing chromedriver; install it or set CHROMEDRIVER=/path/to/chromedriver"; exit 1; }
want=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/"/,"",$3); print $3; exit}' "$CONSUMER_FIXTURE/Cargo.lock")
have=$(wasm-bindgen-test-runner --version | awk '{print $2}')
if [ "$want" != "$have" ]; then
  echo "wasm-bindgen-test-runner $have does not match fixture wasm-bindgen $want"
  exit 1
fi

set -x
mkdir -p "$HYDRATION_OUT"
cd "$CONSUMER_FIXTURE"
CARGO_TARGET_DIR="$HYDRATION_OUT" \
  $CARGO run --locked --example render_counter -- "$HYDRATION_OUT/counter.html"
{ set +x; } 2>/dev/null
test -s "$HYDRATION_OUT/counter.html" || { echo "SSR render produced no HTML"; exit 1; }

binary=''
if [ -n "$CHROME_BIN" ]; then binary=",\"binary\":\"$CHROME_BIN\""; fi
printf '{"goog:chromeOptions":{"args":["--headless=new","--no-sandbox","--disable-gpu","--disable-dev-shm-usage"]%s}}\n' "$binary" \
  > "$HYDRATION_OUT/webdriver.json"

set -x
RANGULAR_SSR_HTML="$HYDRATION_OUT/counter.html" \
CHROMEDRIVER="$(command -v "$CHROMEDRIVER")" \
WASM_BINDGEN_TEST_WEBDRIVER_JSON="$HYDRATION_OUT/webdriver.json" \
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner \
CARGO_TARGET_DIR="$HYDRATION_OUT" \
  $CARGO test --locked --target wasm32-unknown-unknown --no-default-features --features hydrate --test hydration
RANGULAR_SSR_HTML="$HYDRATION_OUT/counter.html" CARGO_TARGET_DIR="$HYDRATION_OUT" \
  $CARGO clippy --locked --target wasm32-unknown-unknown --no-default-features --features hydrate --all-targets -- -D warnings
