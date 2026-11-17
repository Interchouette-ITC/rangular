# Demo browser SPA image (Render): build, run, push to Docker Hub and GHCR.

.PHONY: docker-build docker-build-dev docker-run \
	docker-push-dev-hub docker-push-dev-ghcr-personal docker-push-dev-ghcr-itc \
	docker-push-release-hub docker-push-release-ghcr-personal docker-push-release-ghcr-itc

DOCKER_PORT ?= 8080
DOCKER_BUILDKIT ?= 1
DOCKERFILE ?= docker/Dockerfile
HUB_IMAGE := interchouette/rangular-demo
GHCR_PERSONAL_IMAGE := ghcr.io/groussac/rangular-demo
GHCR_WORKER_IMAGE := ghcr.io/interchouette/rangular-demo
GHCR_ORG_IMAGE := ghcr.io/interchouette-itc/rangular-demo

docker-build:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build --pull --network=host \
		-f $(DOCKERFILE) -t $(HUB_IMAGE):$(APP_VERSION) $(ROOT)
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(HUB_IMAGE):latest

docker-build-dev:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build --pull --network=host \
		-f $(DOCKERFILE) -t $(HUB_IMAGE):dev $(ROOT)
	docker tag $(HUB_IMAGE):dev $(HUB_IMAGE):latest
	docker tag $(HUB_IMAGE):dev $(GHCR_PERSONAL_IMAGE):dev
	docker tag $(HUB_IMAGE):dev $(GHCR_PERSONAL_IMAGE):latest
	docker tag $(HUB_IMAGE):dev $(GHCR_WORKER_IMAGE):dev
	docker tag $(HUB_IMAGE):dev $(GHCR_WORKER_IMAGE):latest
	docker tag $(HUB_IMAGE):dev $(GHCR_ORG_IMAGE):dev
	docker tag $(HUB_IMAGE):dev $(GHCR_ORG_IMAGE):latest

docker-run: docker-build-dev
	@if ss -tlnp 2>/dev/null | grep -q ':$(DOCKER_PORT) '; then \
		echo "Port $(DOCKER_PORT) already in use"; \
		exit 1; \
	fi
	docker run --rm -p $(DOCKER_PORT):8080 -e PORT=8080 $(HUB_IMAGE):latest

docker-push-dev-hub:
	docker push $(HUB_IMAGE):dev
	docker push $(HUB_IMAGE):latest

docker-push-dev-ghcr-personal:
	docker push $(GHCR_PERSONAL_IMAGE):dev
	docker push $(GHCR_PERSONAL_IMAGE):latest

docker-push-dev-ghcr-itc:
	docker push $(GHCR_WORKER_IMAGE):dev
	docker push $(GHCR_WORKER_IMAGE):latest
	docker push $(GHCR_ORG_IMAGE):dev
	docker push $(GHCR_ORG_IMAGE):latest

docker-push-release-hub:
	docker push $(HUB_IMAGE):$(APP_VERSION)
	docker push $(HUB_IMAGE):latest

docker-push-release-ghcr-personal:
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_PERSONAL_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_PERSONAL_IMAGE):latest
	docker push $(GHCR_PERSONAL_IMAGE):$(APP_VERSION)
	docker push $(GHCR_PERSONAL_IMAGE):latest

docker-push-release-ghcr-itc:
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_WORKER_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_WORKER_IMAGE):latest
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_ORG_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_ORG_IMAGE):latest
	docker push $(GHCR_WORKER_IMAGE):$(APP_VERSION)
	docker push $(GHCR_WORKER_IMAGE):latest
	docker push $(GHCR_ORG_IMAGE):$(APP_VERSION)
	docker push $(GHCR_ORG_IMAGE):latest
