FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release --examples

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/examples/grid_stability /usr/local/bin/grid_stability
COPY --from=builder /app/target/release/examples/viral_campaign /usr/local/bin/viral_campaign

ENV RUST_LOG=info

ENTRYPOINT ["grid_stability"]