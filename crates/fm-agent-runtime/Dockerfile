# Build stage
FROM rust:1.87-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release -p fm-agent-runtime

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/fm-agent-runtime /app/fm-agent-runtime

# Non-root user
RUN groupadd -g 1000 forgemaster && \
    useradd -u 1000 -g forgemaster -s /bin/sh forgemaster

USER forgemaster

ENV RUST_LOG=info

ENTRYPOINT ["/app/fm-agent-runtime"]
