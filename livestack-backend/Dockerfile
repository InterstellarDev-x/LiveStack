# syntax=docker/dockerfile:1.7
#
# Builds every LiveStack backend binary (api, producer, consumer,
# email-worker, webhook-worker) plus a one-shot `migrate` image from a single
# Dockerfile. Select which one you want with `--target <name>`; docker-compose
# does this per service.

########################################
# chef: shared builder base + native build deps
########################################
FROM rust:1-slim-bookworm AS chef
WORKDIR /app
# build-essential: cc/make for curl/libz/openssl-sys vendored builds
# cmake: required by pq-sys's bundled libpq build
# perl: required by openssl-sys's vendored build
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential cmake perl pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

########################################
# planner: compute a dependency-only build recipe
########################################
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

########################################
# builder: compile deps (cached layer), then the workspace binaries
########################################
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release \
        --bin api --bin producer --bin consumer --bin email-worker --bin webhook-worker

########################################
# runtime-base: minimal shared base for the service images
########################################
FROM debian:bookworm-slim AS runtime-base
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser
USER appuser
WORKDIR /app

FROM runtime-base AS api
COPY --from=builder /app/target/release/api ./api
EXPOSE 3000
ENTRYPOINT ["./api"]

FROM runtime-base AS producer
COPY --from=builder /app/target/release/producer ./producer
ENTRYPOINT ["./producer"]

FROM runtime-base AS consumer
COPY --from=builder /app/target/release/consumer ./consumer
ENTRYPOINT ["./consumer"]

FROM runtime-base AS email-worker
COPY --from=builder /app/target/release/email-worker ./email-worker
ENTRYPOINT ["./email-worker"]

FROM runtime-base AS webhook-worker
COPY --from=builder /app/target/release/webhook-worker ./webhook-worker
ENTRYPOINT ["./webhook-worker"]

########################################
# migrate: one-shot diesel CLI runner, applies store/migrations and exits
########################################
FROM chef AS migrate
RUN cargo install diesel_cli --no-default-features --features postgres --locked
WORKDIR /app/store
COPY store/diesel.toml ./diesel.toml
COPY store/migrations ./migrations
ENTRYPOINT ["diesel", "migration", "run"]
