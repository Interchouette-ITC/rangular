# Version helpers (workspace + demos + tauri.conf + install snippets + lockfiles).
# Release via GitHub Release tag v$(APP_VERSION). Heavy body: make/version/version-apply.sh.

.PHONY: version-show version-bump-patch version-bump-minor version-bump-major version-set

## Docs whose `rangular = "x.y.z"` / `rangular-build = "x.y.z"` snippets follow the version.
VERSIONED_DOCS := docs/README.md crates/rangular/README.md crates/rangular-build/README.md

VERSION_APPLY = CARGO="$(CARGO)" CONSUMER_FIXTURE="$(CONSUMER_FIXTURE)" VERSIONED_DOCS="$(VERSIONED_DOCS)" \
	$(ROOT)/make/version/version-apply.sh "$(APP_VERSION)"

version-show:
	@echo "Current version: $(APP_VERSION)"; \
	echo "demo-leptos:     $$(awk '/^version = /{gsub(/"/,"",$$3); print $$3; exit}' demo-leptos/Cargo.toml)"; \
	echo "demo-tauri:      $$(awk '/^version = /{gsub(/"/,"",$$3); print $$3; exit}' demo-tauri/src/Cargo.toml)"; \
	echo "tauri.conf.json: $$(python3 -c "import json; print(json.load(open('demo-tauri/src/tauri.conf.json'))['version'])")"; \
	echo ""; \
	echo "Suggested GitHub Release tag:"; \
	echo "  v$(APP_VERSION)"; \
	echo ""; \
	echo "When creating a GitHub Release, use the Tag field (not only the title)."

version-bump-patch:
	@$(VERSION_APPLY) "$(shell echo "$(APP_VERSION)" | awk -F. '{print $$1"."$$2"."($$3+1)}')"

version-bump-minor:
	@$(VERSION_APPLY) "$(shell echo "$(APP_VERSION)" | awk -F. '{print $$1"."($$2+1)".0"}')"

version-bump-major:
	@$(VERSION_APPLY) "$(shell echo "$(APP_VERSION)" | awk -F. '{print ($$1+1)".0.0"}')"

version-set:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make version-set VERSION=x.y.z"; \
		exit 1; \
	fi
	@$(VERSION_APPLY) "$(VERSION)"
