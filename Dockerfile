FROM rust:1.90-bullseye AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches

RUN cargo fetch
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/rustcord /app/rustcord

CMD ["/app/rustcord"]
