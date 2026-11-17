# crates.io packaging. Heavy body: make/publish/publish-crates.sh (order, skip, index wait).
# Upload runs from the release workflow: make/publish/publish-crates.sh publish.

.PHONY: publish-check

## Packages and verifies all nine crates (local registry, no upload).
publish-check:
	CARGO="$(CARGO)" $(ROOT)/make/publish/publish-crates.sh check $(PUBLISH_CHECK_ARGS)
