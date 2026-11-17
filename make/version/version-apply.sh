#!/usr/bin/env bash
# Moves every version location from <current> to <new>:
#   1. [workspace.package] + workspace dependency versions (Cargo.toml).
#   2. demo-leptos and demo-tauri package versions; tauri.conf.json.
#   3. `rangular = "x.y.z"` / `rangular-build = "x.y.z"` install snippets ($VERSIONED_DOCS).
#   4. Lockfiles: workspace, both demos, consumer fixture (fails if one cannot refresh).
#
#   make/version/version-apply.sh <current> <new>
#
# Operator entry: `make version-set VERSION=x.y.z` / `make version-bump-patch|minor|major`.
# Env: CARGO, CONSUMER_FIXTURE, VERSIONED_DOCS (space-separated, relative to the repo root).
set -euo pipefail

ROOT="$(cd -- "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

current=${1:-}
new=${2:-}
CARGO=${CARGO:-cargo}
CONSUMER_FIXTURE=${CONSUMER_FIXTURE:-$ROOT/crates/rangular-build/tests/fixtures/document-consumer}
VERSIONED_DOCS=${VERSIONED_DOCS:-docs/README.md crates/rangular/README.md crates/rangular-build/README.md}

if [ -z "$current" ]; then echo "empty current version"; exit 1; fi
if [ -z "$new" ]; then echo "empty version"; exit 1; fi

sed -i "s/version = \"$current\"/version = \"$new\"/g" Cargo.toml
sed -i "s/^version = \"$current\"/version = \"$new\"/" demo-leptos/Cargo.toml
sed -i "s/^version = \"$current\"/version = \"$new\"/" demo-tauri/src/Cargo.toml
# shellcheck disable=SC2086 # VERSIONED_DOCS is a space-separated list.
sed -i -E "s/^(rangular(-build)? = )\"$current\"/\1\"$new\"/" $VERSIONED_DOCS
python3 -c "import json, sys; p='demo-tauri/src/tauri.conf.json'; c=json.load(open(p)); c['version']=sys.argv[1]; json.dump(c, open(p,'w'), indent=2); open(p,'a').write('\n')" "$new"

for manifest in Cargo.toml demo-leptos/Cargo.toml demo-tauri/src/Cargo.toml "$CONSUMER_FIXTURE/Cargo.toml"; do
  $CARGO metadata --manifest-path "$manifest" --format-version 1 >/dev/null \
    || { echo "failed to refresh the lockfile of $manifest"; exit 1; }
done

echo "Version $current → $new (workspace + demos + tauri.conf + install snippets)"
