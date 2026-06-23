FROM rust:1.96-bullseye AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY packages ./packages

RUN cargo fetch
RUN cargo build --release -p sync
RUN cargo install sqlx-cli --no-default-features --features postgres

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/sync /app/sync
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations ./migrations
ENTRYPOINT ["/app/sync"]
