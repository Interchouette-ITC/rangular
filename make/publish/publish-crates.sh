#!/usr/bin/env bash
# Package or publish the workspace crates to crates.io in dependency order.
#
#   make/publish/publish-crates.sh check    package + verify all crates locally
#   make/publish/publish-crates.sh publish  upload versions not yet on crates.io
#
# Operator entry: `make publish-check`; upload runs from the release workflow.
#
# `publish` is safe to rerun after a partial failure: versions already on
# crates.io are not uploaded again, but each one is still awaited in the
# sparse index before its dependents are published. It needs CARGO_REGISTRY_TOKEN and never passes
# --no-verify.
set -euo pipefail

CARGO=${CARGO:-cargo}
INDEX_ATTEMPTS=${INDEX_ATTEMPTS:-60}
INDEX_INTERVAL=${INDEX_INTERVAL:-10}
USER_AGENT="rangular-release (https://github.com/Interchouette-ITC/rangular)"
# Bounds every request so a stalled connection cannot hang the release.
CURL_TIMEOUTS=(--connect-timeout 10 --max-time 30)

# Dependencies before dependents (normal and versioned dev-dependencies).
CRATES=(
  rangular-host
  rangular-expr
  rangular-parser
  rangular-css
  rangular-macros
  rangular-aot
  rangular-runtime
  rangular-build
  rangular
)

cd "$(dirname "$0")/../.."

workspace_version() {
  awk '/^\[workspace.package\]/{p=1;next} p&&/^version = /{gsub(/"/,"",$3); print $3; exit}' Cargo.toml
}

# Fails if the publish list and the workspace members drift apart or if a
# member does not carry the workspace version.
check_members() {
  local version=$1
  $CARGO metadata --no-deps --format-version 1 --locked |
    python3 -c '
import json, sys
version, expected = sys.argv[1], set(sys.argv[2:])
packages = {p["name"]: p for p in json.load(sys.stdin)["packages"]}
if set(packages) != expected:
    sys.exit(f"publish list {sorted(expected)} != workspace {sorted(packages)}")
for name, package in sorted(packages.items()):
    if package["version"] != version:
        sys.exit(name + " is " + package["version"] + ", expected " + version)
    if package.get("publish") == []:
        sys.exit(f"{name} has publish = false")
' "$version" "${CRATES[@]}"
}

# Prints 200 if the version exists on crates.io, 404 if not.
registry_status() {
  curl "${CURL_TIMEOUTS[@]}" -sS -o /dev/null -w '%{http_code}' -A "$USER_AGENT" \
    "https://crates.io/api/v1/crates/$1/$2"
}

wait_for_index() {
  local name=$1 version=$2 attempt body
  local url="https://index.crates.io/${name:0:2}/${name:2:2}/${name}"
  for ((attempt = 1; attempt <= INDEX_ATTEMPTS; attempt++)); do
    # Read the whole entry first: piping into `grep -q` under pipefail can fail
    # with SIGPIPE even when the version is present.
    body=$(curl "${CURL_TIMEOUTS[@]}" -fsS -A "$USER_AGENT" "$url" 2>/dev/null) || body=''
    if [[ $body == *"\"vers\":\"${version}\""* ]]; then
      echo "$name $version is available in the index"
      return 0
    fi
    sleep "$INDEX_INTERVAL"
  done
  echo "error: $name $version not in the crates.io index after $((INDEX_ATTEMPTS * INDEX_INTERVAL))s" >&2
  return 1
}

publish() {
  local version=$1 name status
  if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
    echo "error: CARGO_REGISTRY_TOKEN is not set" >&2
    exit 1
  fi
  for name in "${CRATES[@]}"; do
    status=$(registry_status "$name" "$version")
    case "$status" in
      200)
        # A previous run may have uploaded it but timed out before indexing;
        # dependents must not be published until the index serves it.
        echo "$name $version already on crates.io; skipping upload"
        wait_for_index "$name" "$version"
        continue
        ;;
      404) ;;
      *)
        echo "error: crates.io returned HTTP $status for $name $version" >&2
        exit 1
        ;;
    esac
    echo "Publishing $name $version"
    $CARGO publish -p "$name" --locked
    wait_for_index "$name" "$version"
  done
}

main() {
  local mode=${1:-} version
  version=$(workspace_version)
  [ -n "$version" ] || { echo "error: no [workspace.package] version" >&2; exit 1; }
  check_members "$version"
  case "$mode" in
    check)
      # Verifies every package against a temporary local registry, so
      # dependents build even before their dependencies are on crates.io.
      $CARGO package --workspace --locked "${@:2}"
      ;;
    publish) publish "$version" ;;
    *)
      echo "usage: $0 check [cargo package args] | publish" >&2
      exit 2
      ;;
  esac
}

main "$@"
