# ADR-0006: MCP Client SDK Selection

**Status:** Accepted

**Date:** 2026-02-07

**Decision Makers:** CSM-101

**Technical Area:** Tool Integration, MCP Client

## Context

The agent runtime (`fm-agent-runtime-claude`) needs an MCP client to connect to MCP servers inside the Kubernetes cluster. The runtime discovers tools via `tools/list` and executes them via `tools/call` using JSON-RPC 2.0 over HTTP.

The Anthropic Messages API native MCP connector (`mcp_servers` parameter) requires publicly accessible HTTPS endpoints — it cannot reach local K8s services. Therefore, the runtime itself must act as the MCP client.

Three options are available:

1. **Official Rust MCP SDK (`rmcp`)** — full protocol implementation
2. **Community SDK (`rust-mcp-sdk`)** — alternative implementation
3. **Minimal custom client** — two JSON-RPC POST requests using `reqwest`

## Decision Drivers

- **Protocol correctness** — MCP protocol is evolving, client must stay compatible
- **Maintenance burden** — hackathon timeline, minimize custom protocol code
- **Dependency footprint** — avoid pulling excessive dependencies into the runtime
- **Community adoption** — prefer well-maintained, widely-used crates
- **Future extensibility** — may need MCP resources, prompts, notifications later

## Considered Options

### Option 1: `rmcp` (Official Rust MCP SDK)

The [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk) for MCP. Version 0.14.0, 932K+ monthly downloads, used by 531 crates.

Features needed: `client` + `transport-streamable-http-client` + `transport-streamable-http-client-reqwest`

```toml
rmcp = { version = "0.14", features = ["client", "transport-streamable-http-client", "transport-streamable-http-client-reqwest"] }
```

**Pros:**
- Official SDK maintained by the MCP project
- Full protocol support (tools, resources, prompts, notifications)
- Handles protocol evolution — new MCP versions are supported automatically
- Built-in Streamable HTTP transport using `reqwest` (already in our deps)
- High adoption (932K downloads/month)
- Session management, error handling built-in

**Cons:**
- Adds ~16 direct dependencies to the runtime
- Larger binary size (~261K SLoC across dependency tree)
- May include features we don't use yet (resources, prompts)

### Option 2: `rust-mcp-sdk` (Community SDK)

The [community SDK](https://github.com/rust-mcp-stack/rust-mcp-sdk). Implements MCP protocol version 2025-11-25.

```toml
rust-mcp-sdk = { version = "0.2", features = ["client", "hyper-client"] }
```

**Pros:**
- Full MCP protocol support
- Axum/Hyper-based (aligns with our web framework choice)
- Backward compatibility built-in

**Cons:**
- Not the official SDK
- Smaller community (fewer downloads, less battle-tested)
- Different API patterns than the official SDK
- Risk of diverging from the official MCP spec over time

### Option 3: Minimal Custom Client

Two JSON-RPC 2.0 POST requests using `reqwest` (already in deps). ~150 lines of code.

```rust
// tools/list
let response = http.post(url).json(&json!({"jsonrpc":"2.0","method":"tools/list","id":1})).send().await?;
// tools/call
let response = http.post(url).json(&json!({"jsonrpc":"2.0","method":"tools/call","params":{...},"id":2})).send().await?;
```

**Pros:**
- Zero new dependencies
- Smallest binary size
- Simple, easy to understand
- Full control over the implementation

**Cons:**
- Must maintain JSON-RPC 2.0 protocol code ourselves
- No session management, no protocol negotiation
- Must manually update when MCP protocol changes
- No support for MCP features beyond tools (resources, prompts)
- Risk of protocol bugs (edge cases in JSON-RPC, error handling)

## Decision

**Use Option 1: `rmcp` (Official Rust MCP SDK).**

The official SDK provides protocol correctness, automatic compatibility with MCP spec updates, and built-in Streamable HTTP transport using `reqwest` (already in our deps). The dependency cost is acceptable for a production agent runtime that must reliably communicate with MCP servers.

The minimal custom client (Option 3) would work for the hackathon but creates maintenance debt — every MCP protocol change requires manual updates. The official SDK eliminates this risk.

## Consequences

### Positive

- Protocol correctness guaranteed by the official implementation
- Future MCP features (resources, prompts, notifications) available without code changes
- No custom JSON-RPC 2.0 code to maintain
- High community adoption reduces risk of abandonment

### Negative

- Adds ~16 dependencies to the runtime binary
- Slightly larger binary size
- Must track `rmcp` version updates

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `rmcp` breaking changes | Low | Medium | Pin version, test on upgrades |
| Dependency conflicts with existing crates | Low | Low | Feature flags are additive, `reqwest` is shared |
| SDK too complex for simple use case | Low | Low | Only use `client` + transport features |

## Related

- [ADR-0004: MCP for Tool Integration](0004-mcp-for-tool-integration.md)
- [Architecture: MCP Integration](../arch/08-mcp-integration.md)
- [rmcp — Official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [rmcp on docs.rs](https://docs.rs/rmcp/latest/rmcp/)
- [rust-mcp-sdk — Community SDK](https://github.com/rust-mcp-stack/rust-mcp-sdk)
