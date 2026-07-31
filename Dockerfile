FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY math-core/Cargo.toml math-core/
COPY state-store/Cargo.toml state-store/
COPY agent-core/Cargo.toml agent-core/
COPY config-layer/Cargo.toml config-layer/

RUN mkdir -p math-core/src state-store/src agent-core/src config-layer/src src examples benches && \
    touch math-core/src/lib.rs state-store/src/lib.rs agent-core/src/lib.rs config-layer/src/lib.rs src/lib.rs src/main.rs benches/population_benchmark.rs && \
    cargo fetch

COPY . .

RUN cargo build --release --examples --bin adaptive-agent-framework

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

RUN mkdir -p /app/examples

COPY --from=builder /app/target/release/examples/grid_stability /usr/local/bin/grid_stability
COPY --from=builder /app/target/release/examples/viral_campaign /usr/local/bin/viral_campaign
COPY --from=builder /app/target/release/examples/swarm_management /usr/local/bin/swarm_management
COPY --from=builder /app/target/release/adaptive-agent-framework /usr/local/bin/aaf
COPY config.toml /app/config.toml

ENV RUST_LOG=info
WORKDIR /app

ENTRYPOINT ["aaf"]