# ADR-0009: A2A SDK Selection — a2a-rs Modular Crates

**Status:** Accepted

**Date:** 2026-02-08

**Decision Makers:** CSM-101

**Technical Area:** Agent-to-Agent Communication, SDK Selection

## Context

ADR-0003 chose the A2A Protocol for agent-to-agent communication. We now need a Rust implementation. Similar to how we chose `rmcp` for MCP (ADR-0006), we need an SDK that provides types, server, and client for A2A.

## Considered Options

### Option 1: a2a-rs v0.1.0 (monolithic)

Single crate by emillindfors. Hexagonal architecture with SQLx database support.

**Pros:**
- All-in-one crate
- Database storage adapters (PostgreSQL, MySQL, SQLite)
- WebSocket support

**Cons:**
- v0.1.0 — early, potentially unstable
- Heavy dependencies (SQLx, database drivers) we don't need
- Monolithic — pulls everything even if we only need types

### Option 2: a2a-rs-core + a2a-rs-client + a2a-rs-server v1.0.0 (modular)

Three modular crates following A2A RC 1.0 spec.

**Pros:**
- v1.0.0 — stable release matching A2A RC 1.0 proto spec
- Modular — use only what you need (types, client, server separately)
- Lightweight — no database dependencies
- Simple server trait: implement `MessageHandler` (`handle_message()` + `agent_card()`)
- Built on Axum (already in our stack)
- Client provides `fetch_agent_card()` and `send_message()`

**Cons:**
- No built-in database storage (we use in-memory, which is fine for short-lived agent pods)

### Option 3: Build from scratch

Implement A2A types, server, and client ourselves.

**Pros:**
- Full control
- No external dependencies

**Cons:**
- Significant implementation effort
- Must track A2A spec changes manually
- Reinventing the wheel

## Decision

**We choose Option 2: a2a-rs modular crates (core + client + server) v1.0.0.**

| Crate | Purpose | Usage |
|-------|---------|-------|
| `a2a-rs-core` | Types (AgentCard, Task, Message, Part) | Both runtime and controller |
| `a2a-rs-server` | Axum-based A2A server | Agent runtime (each agent serves A2A) |
| `a2a-rs-client` | HTTP client for A2A | Agent runtime (agents call peer A2A endpoints) |

This mirrors our MCP approach: `rmcp` provides types + server + client for MCP, `a2a-rs-*` provides the same for A2A.

## Consequences

### Positive

- **Spec-compliant** — follows A2A RC 1.0 proto spec
- **Minimal trait** — only need to implement `handle_message()` and `agent_card()`
- **Modular** — controller only needs `a2a-rs-core` for types
- **Consistent** — same pattern as rmcp for MCP

### Negative

- **External dependency** — must track upstream updates
- **Axum version** — server uses Axum 0.7, our workspace uses 0.8 (may need compatibility check)

## Related

- [ADR-0003: A2A for Agent Communication](0003-a2a-for-agent-communication.md)
- [ADR-0006: MCP Client SDK Selection](0006-mcp-client-sdk-selection.md)
- [A2A Protocol Architecture](../arch/07-a2a-protocol.md)
