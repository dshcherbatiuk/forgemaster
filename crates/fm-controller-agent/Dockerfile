# Build stage
FROM rust:1.87-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/fm-core ./crates/fm-core
COPY crates/fm-controller-agenttask ./crates/fm-controller-agenttask
COPY crates/fm-controller-agent ./crates/fm-controller-agent

# Build release binary
RUN cargo build --release -p fm-controller-agent

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/fm-controller-agent /app/fm-controller-agent

# Non-root user
RUN groupadd -g 1000 forgemaster && \
    useradd -u 1000 -g forgemaster -s /bin/sh forgemaster

USER forgemaster

ENV RUST_LOG=info

ENTRYPOINT ["/app/fm-controller-agent"]
