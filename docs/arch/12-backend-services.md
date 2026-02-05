## Backend Services: Rust

All backend services are written in **Rust** for performance, safety, and low resource footprint.

### Rust Workspace Structure

```
meta-agent/
├── Cargo.toml                 # Workspace root
├── Cargo.lock
├── rust-toolchain.toml
├── .cargo/
│   └── config.toml
│
├── crates/
│   ├── meta-agent-core/       # Shared types, traits, utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs
│   │       ├── error.rs
│   │       └── config.rs
│   │
│   ├── tcp-controller/        # TCP Controller service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── controller.rs
│   │       ├── feedback.rs
│   │       └── api.rs
│   │
│   ├── agent-registry/        # Agent Registry service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── registry.rs
│   │       ├── health.rs
│   │       └── api.rs
│   │
│   ├── agent-runtime/         # Agent execution runtime
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── agent.rs
│   │       ├── llm.rs
│   │       └── executor.rs
│   │
│   ├── a2a-rs/                # A2A Protocol implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── server.rs
│   │       ├── types.rs
│   │       └── transport.rs
│   │
│   ├── a2ui-rs/               # A2UI Protocol implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── components.rs
│   │       ├── renderer.rs
│   │       └── streaming.rs
│   │
│   ├── mcp-client/            # MCP Client implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       └── tools.rs
│   │
│   └── k8s-operator/          # Kubernetes Operator (kube-rs)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── crds.rs
│           ├── controllers/
│           │   ├── mod.rs
│           │   ├── agent_task.rs
│           │   ├── agent.rs
│           │   └── mcp_server.rs
│           └── reconcilers.rs
│
├── docker/
│   ├── Dockerfile.tcp-controller
│   ├── Dockerfile.agent-registry
│   ├── Dockerfile.agent-runtime
│   └── Dockerfile.k8s-operator
│
└── tests/
    ├── integration/
    └── e2e/
```

### Workspace Cargo.toml

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "crates/meta-agent-core",
    "crates/tcp-controller",
    "crates/agent-registry",
    "crates/agent-runtime",
    "crates/a2a-rs",
    "crates/a2ui-rs",
    "crates/mcp-client",
    "crates/k8s-operator",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "Apache-2.0"
repository = "https://github.com/csm-101/meta-agent"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Web framework
axum = { version = "0.7", features = ["ws", "macros"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP client
reqwest = { version = "0.11", features = ["json", "stream"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Kubernetes
kube = { version = "0.87", features = ["runtime", "derive"] }
k8s-openapi = { version = "0.20", features = ["v1_28"] }

# LLM
async-openai = "0.18"  # Works with Claude API

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Config
config = "0.14"
dotenvy = "0.15"

# Testing
tokio-test = "0.4"
wiremock = "0.5"
```

TCP Controller is implemented as an Axum-based HTTP service exposing endpoints for task management, feedback submission, and control signal computation. It maintains context history for the PID-like feedback loop.

Agent Registry Service uses Redis for storage with TTL-based leases, skill indexing for fast lookups, and background health checking.

A2A Protocol implementation provides client and server libraries for agent-to-agent communication using JSON-RPC 2.0 over HTTP with SSE streaming support.

Kubernetes Operator built with kube-rs defines CRDs for AgentTask, Agent, and MCPServer resources. Controllers reconcile these resources to manage the agent lifecycle within the cluster.

### Dockerfile (Multi-stage Build)

```dockerfile
# docker/Dockerfile.tcp-controller

# Build stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --package tcp-controller

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tcp-controller /usr/local/bin/

ENV RUST_LOG=tcp_controller=info

EXPOSE 8080

CMD ["tcp-controller"]
```

### Makefile for Rust

```makefile
# Makefile

.PHONY: build test lint fmt check docker-build

# Build all crates
build:
	cargo build --release

# Run tests
test:
	cargo test --workspace

# Run lints
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Check formatting and lints
check: fmt lint
	cargo check --workspace

# Build Docker images
docker-build:
	docker build -f docker/Dockerfile.tcp-controller -t metaagent/tcp-controller:latest .
	docker build -f docker/Dockerfile.agent-registry -t metaagent/agent-registry:latest .
	docker build -f docker/Dockerfile.agent-runtime -t metaagent/agent-runtime:latest .
	docker build -f docker/Dockerfile.k8s-operator -t metaagent/k8s-operator:latest .

# Run locally
run-tcp-controller:
	cargo run --package tcp-controller

run-registry:
	cargo run --package agent-registry

# Generate CRD manifests
generate-crds:
	cargo run --package k8s-operator -- generate-crds > helm/meta-agent/crds/
```
