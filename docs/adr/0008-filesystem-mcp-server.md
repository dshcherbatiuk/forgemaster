# ADR-0008: Filesystem MCP Server for Agent Workspace Access

**Status:** Accepted

**Date:** 2026-02-08

**Decision Makers:** CSM-101

**Technical Area:** MCP, Agent Infrastructure

## Context

Agents (code-generator, test-generator, reviewer) need to read and write files in the shared workspace directory (`/workspace`). Currently agents produce output as text in their conversation response, but cannot persist files to disk. The orchestrator relays outputs between agents via `task_prompt`, but this doesn't scale for large codebases and prevents agents from working with real files.

A filesystem MCP server would give agents tools like `read_file`, `write_file`, `list_directory` — enabling them to work with the shared workspace as a real filesystem.

The agent runtime already supports MCP via rmcp SDK with Streamable HTTP transport. The `McpServerRef` convention expects servers at `http://<name>.<namespace>.svc.cluster.local:<port>/mcp`.

## Decision Drivers

- **Time to deploy** — hackathon timeline, must be deployable immediately
- **Proven implementation** — use battle-tested official tooling, not custom code
- **Streamable HTTP** — agents connect via Streamable HTTP transport
- **Security** — path sandboxing to restrict access to the workspace directory only
- **Simplicity** — wrap existing tools, don't reinvent

## Considered Options

### Option 1: Official Node.js server + supergateway bridge

The [official `@modelcontextprotocol/server-filesystem`](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) is stdio-only. Wrap with [supergateway](https://github.com/supercorp-ai/supergateway) to expose Streamable HTTP.

Deploy as a K8s Service per task namespace. Agents connect via `MCP_SERVER_URLS`.

```
supergateway --stdio "npx -y @modelcontextprotocol/server-filesystem /workspace" \
  --port 3000 --outputTransport streamableHttp
```

**Pros:**
- Official implementation, 13 tools, well-tested
- supergateway handles Streamable HTTP bridging
- Zero custom filesystem code
- Ready to deploy immediately (npm packages)
- Built-in path sandboxing (only allowed directories accessible)

**Cons:**
- Requires Node.js runtime (~250MB image)
- Two processes per pod (supergateway + filesystem server)
- Inconsistent with Rust codebase
- Extra dependency (npm, Node.js)

### Option 2: Go server fork with Streamable HTTP

Fork [mark3labs/mcp-filesystem-server](https://github.com/mark3labs/mcp-filesystem-server) (Go, 14 tools). The underlying `mcp-go` library supports Streamable HTTP natively — change `ServeStdio()` to `ServeStreamableHTTP()`.

**Pros:**
- Small binary (~20MB image)
- Native Streamable HTTP via `mcp-go`

**Cons:**
- Go dependency in a Rust project
- Must maintain a fork
- Different build toolchain

### Option 3: Existing Rust crate (`filesystem-mcp-rs`)

[filesystem-mcp-rs](https://docs.rs/crate/filesystem-mcp-rs/0.1.7) (v0.1.7) — Rust port of the official server with 27 tools.

**Pros:**
- Rust, small binary, feature-rich

**Cons:**
- Uses SSE transport, not Streamable HTTP
- Does not use `rmcp` SDK
- Early stage (v0.1.7)

### Option 4: Build own with rmcp SDK

Build a custom filesystem MCP server in Rust using rmcp SDK.

**Pros:**
- Same SDK and patterns as existing code
- Smallest image (~15MB)

**Cons:**
- Must implement and test ~8-10 filesystem tools
- Must implement path sandboxing (security-critical)
- Development time: 1-2 days — too much for hackathon timeline

## Decision

**Use Option 1: Official Node.js filesystem server + supergateway bridge.**

The hackathon timeline does not allow building a custom filesystem MCP server. The official implementation is battle-tested with 13 tools, built-in path sandboxing, and active maintenance. supergateway bridges stdio to Streamable HTTP, which is exactly what the rmcp client expects.

The ~250MB image size is acceptable for the hackathon. Post-hackathon, this can be replaced with a custom Rust implementation (Option 4) if image size or consistency becomes a concern.

## Implementation

### Docker image

Single Dockerfile that bundles Node.js + supergateway + filesystem server:

```dockerfile
FROM node:22-slim
RUN npm install -g @anthropic-ai/supergateway @modelcontextprotocol/server-filesystem
EXPOSE 3000
ENTRYPOINT ["supergateway", \
  "--stdio", "npx -y @modelcontextprotocol/server-filesystem /workspace", \
  "--port", "3000", \
  "--outputTransport", "streamableHttp"]
```

### K8s deployment

- Deploy as a **K8s Deployment + Service** per task namespace
- Service name: `fm-mcp-filesystem` — agents connect via `http://fm-mcp-filesystem.<namespace>.svc.cluster.local:3000/mcp`
- Mount the same workspace hostPath volume as agent pods
- Helm chart in `crates/fm-mcp-filesystem/helm/`
- Ansible role for build + deploy

### Agent wiring

- Add `fm-mcp-filesystem` to agent `McpServerRef` list (alongside `fm-controller-agent-mcp`)
- Agent Controller creates the filesystem MCP server pod in the task namespace (during Pending → Running transition, or via orchestrator factory)
- Agents discover filesystem tools via `tools/list` at startup

### Available tools (13)

| Tool | Description |
|------|-------------|
| `read_text_file` | Read file contents as text |
| `read_media_file` | Read image/audio as base64 |
| `read_multiple_files` | Read multiple files at once |
| `write_file` | Create or overwrite a file |
| `edit_file` | Pattern-matching selective edits |
| `create_directory` | Create directory with parents |
| `list_directory` | List contents with type prefixes |
| `list_directory_with_sizes` | List with sizes and sorting |
| `move_file` | Move or rename |
| `search_files` | Recursive glob search |
| `directory_tree` | Recursive JSON tree |
| `get_file_info` | File metadata |
| `list_allowed_directories` | List sandboxed directories |

## Consequences

### Positive

- Agents can read/write files in the shared workspace immediately
- Zero custom filesystem code to maintain
- Official implementation with active maintenance
- Built-in path sandboxing (only `/workspace` accessible)
- 13 tools available out of the box

### Negative

- Node.js runtime dependency (~250MB image)
- Two processes per pod (supergateway + filesystem server)
- Inconsistent tech stack (Node.js in a Rust project)
- supergateway is an additional dependency to track

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| supergateway Streamable HTTP incompatible with rmcp client | Low | High | Test connectivity before deploying to agents; both follow MCP spec |
| Node.js image too large for cluster resources | Low | Low | Acceptable for hackathon; replace with Rust post-hackathon |
| File conflicts between concurrent agents | Low | Medium | Agents execute sequentially (orchestrator controls ordering) |
| supergateway or filesystem server breaking update | Low | Medium | Pin npm versions in Dockerfile |

## Related

- [ADR-0004: MCP for Tool Integration](0004-mcp-for-tool-integration.md)
- [ADR-0006: MCP Client SDK Selection](0006-mcp-client-sdk-selection.md)
- [ADR-0007: Per-Agent-Type Prompt Guidelines](0007-per-agent-type-prompt-injection.md)
- Agent Controller MCP server: `crates/fm-controller-agent/src/mcp_server/`
- [Official Filesystem MCP Server](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem)
- [supergateway](https://github.com/supercorp-ai/supergateway)
- [rmcp SDK](https://docs.rs/rmcp/latest/rmcp/)
