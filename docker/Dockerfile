# syntax=docker/dockerfile:1

# Browser demo: Trunk + Leptos CSR wasm → nginx (Render / Hub image).
# Build context: repository root.
#
#   docker build -f docker/Dockerfile -t interchouette/rangular-demo .
#   make docker-run

ARG RUST_IMAGE=rust:1.88-bookworm
ARG NGINX_IMAGE=nginxinc/nginx-unprivileged:1.27-alpine

FROM ${RUST_IMAGE} AS builder

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    clang \
    pkg-config \
  && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown \
  && cargo install --locked trunk --version 0.21.14

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY tests/fixtures ./tests/fixtures
COPY docs/assets ./docs/assets
COPY demo ./demo

ENV CARGO_TERM_COLOR=never \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN cd demo && trunk build --release

FROM ${NGINX_IMAGE} AS runtime

COPY docker/nginx.conf /etc/nginx/nginx.conf
COPY docker/entrypoint.sh /entrypoint.sh
COPY --from=builder /app/demo/dist /usr/share/nginx/html

RUN chmod +x /entrypoint.sh

USER nginx
EXPOSE 8080

ENTRYPOINT ["/entrypoint.sh"]
CMD ["nginx", "-g", "daemon off;"]
