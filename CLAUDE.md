# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ForgeMAster** is a meta-agent system for the AgentForge Hackathon 2026. It implements autonomous agent orchestration using a TCP (Task-Context-Prediction) controller inspired by PID control theory. The system dynamically provisions AI agents and MCP servers, using E2E tests (Gherkin) as the setpoint for measuring success.

## Architecture

### Core Design Principle
- **TCP Controller**: Pure Rust + math (no LLM) - handles orchestration decisions (fast, deterministic, free)
- **Agents**: Claude API - handles creative work (code generation, test writing, reviewing)

### Key Components
1. **TCP Controller** (`tcp-controller`) - PID-like controller computing control signals from error rates
2. **Agent Registry** (`agent-registry`) - Service discovery with registration, heartbeat, skill-based search
3. **Agent Runtime** (`agent-runtime`) - LLM agent execution with Claude API
4. **A2A Protocol** (`a2a-rs`) - Agent-to-agent communication
5. **K8s Operator** (`k8s-operator`) - CRD controllers for AgentTask, Agent, MCPServer, TestSuite

### Feedback Loop
```
Task Input → Test Generator (Gherkin) → Executor Agents → Test Runner → Feedback Collector → TCP Controller
                                              ↑                                    │
                                              └────────────────────────────────────┘
```

### Error Signal
```
error = failed_tests / total_tests
```

Control actions based on error:
- High (>0.5): Swap agent, change approach
- Medium (0.2-0.5): Add reviewer agent, adjust params
- Low (<0.2): Fine-tune, retry failed tests

## Build & Test Commands

```bash
# Build (release mode first as per CLAUDE.md rules)
cargo build --release
cargo test

# Run individual crates
cargo run --package tcp-controller
cargo run --package agent-registry

# Run single test
cargo test --package <crate-name> <test_name>

# Run workspace tests
cargo test --workspace

# Docker build
docker build -f docker/Dockerfile.tcp-controller -t tcp-controller .

# Deploy to Kind
kind create cluster --name meta-agent
helm install meta-agent ./helm/meta-agent
```

## Project Structure

```
meta-agent/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── meta-agent-core/    # Shared types, error handling
│   ├── tcp-controller/     # PID computation, REST API (Axum), Redis
│   ├── agent-registry/     # Register/deregister, skill search, health checks
│   ├── agent-runtime/      # Claude API integration, streaming
│   ├── a2a-rs/             # A2A protocol implementation
│   └── k8s-operator/       # CRD controllers
├── helm/meta-agent/        # Helm chart
├── ansible/playbooks/      # Local deployment
└── docker/Dockerfile.*     # Container builds
```

## Custom Resource Definitions (CRDs)

- `AgentTask` - Top-level task definition with TCP controller settings
- `Agent` - Individual agent instance (LLM config, system prompt, MCP servers)
- `MCPServer` - MCP server instance (github, filesystem, prometheus)
- `TestSuite` - Gherkin test suite (the setpoint)

## Key Rust Patterns

TCP Controller signal computation:
```rust
let signal = Kt * error + Kc * integral + Kp * derivative;
match signal {
    s if s > 0.5 => SwapAgent,
    s if s > 0.2 => AddAgent,
    _ => Continue,
}
```

## Dependencies

- **Web Framework**: Axum
- **State/Cache**: Redis
- **LLM Provider**: Anthropic Claude API
- **Container Orchestration**: Kubernetes with custom operators
- **Test Framework**: Gherkin/BDD (behave, cucumber)
