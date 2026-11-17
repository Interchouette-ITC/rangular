# rangular developer targets.
# Thin index: recipes live under make/ (folders hold .mk + companion .sh). Run `make help`.
#
#   make/common.mk            - shared variables
#   make/quality.mk           - check / test / no-panic / format / lint / clean / ci; includes make/quality/
#   make/quality/audit.mk     - audit / deny / machete / outdated / geiger
#   make/quality/coverage.mk  - coverage / coverage-summary / coverage-html / tarpaulin
#   make/quality/fuzz.mk      - fuzz / fuzz-build
#   make/quality/doc.mk       - doc / doc-open / doc-clean
#   make/hydration/           - hydration.mk + test-hydration.sh (SSR → browser hydration)
#   make/publish/             - publish.mk + publish-crates.sh (crates.io)
#   make/demo.mk              - demo-leptos (Trunk) / demo-tauri
#   make/docker.mk            - demo image build / run / push
#   make/version/             - version.mk + version-apply.sh
#   make/help.mk              - target catalogue (default goal)

include make/common.mk
include make/quality.mk
include make/hydration/hydration.mk
include make/publish/publish.mk
include make/demo.mk
include make/docker.mk
include make/version/version.mk
include make/help.mk

.DEFAULT_GOAL := help
