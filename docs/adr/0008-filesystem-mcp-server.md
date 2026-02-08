# ADR-0008: Filesystem MCP Server for Agent Workspace Access

**Status:** Superseded (switched from Option 1 to Option 4)

**Date:** 2026-02-08

**Decision Makers:** CSM-101

**Technical Area:** MCP, Agent Infrastructure

## Context

Agents (code-generator, test-generator, reviewer) need to read and write files in the shared workspace directory (`/workspace`). A filesystem MCP server gives agents tools like `read_file`, `write_file`, `list_directory`.

The agent runtime connects to MCP servers via rmcp SDK with Streamable HTTP transport. The `McpServerRef` convention expects servers at `http://<name>.<namespace>.svc.cluster.local:<port>/mcp`.

### Why Option 1 was replaced

The original decision (Node.js supergateway + @modelcontextprotocol/server-filesystem) proved unstable in production:

- **supergateway closes SSE streams after each response** — rmcp client interprets this as a fatal transport error, killing the connection permanently
- **Single stdio pipe bottleneck** — `--stateful` mode processes requests sequentially; concurrent agents cause timeouts and "empty sse stream" errors
- **Session management failures** — 400 errors on session cleanup
- **Unreliable health checks** — `/healthz` served by supergateway itself, not the child process; K8s thinks the pod is healthy even when the filesystem server has crashed
- **Large image size** — ~250MB for Node.js runtime

These issues caused cascading failures: orchestrator agents crash 3+ times per task due to MCP connection errors, creating duplicate agents and losing pipeline progress.

## Decision

**Use Option 4: Native Rust MCP server using rmcp SDK.**

Same SDK, same transport, same patterns as `fm-mcp-devtools` and `fm-controller-agent`. Zero transport translation layers.

## Implementation

### Architecture

```
rmcp client (agent) → Streamable HTTP → fm-mcp-filesystem (Rust, rmcp server SDK)
```

No supergateway, no stdio bridge, no Node.js.

### Crate structure

`crates/fm-mcp-filesystem/` — follows the same pattern as `fm-mcp-devtools`:
- `handler.rs` — `FilesystemHandler` with `#[tool_router]` and `#[tool_handler]` macros
- `param/` — one param struct per tool (Deserialize + JsonSchema)
- `action/` — one action module per tool (validates workspace path, performs I/O)
- `workspace.rs` — path sandboxing (all paths must be under `WORKSPACE_ROOT`)
- `lib.rs` — server startup with `StreamableHttpService` at `/mcp` + `/healthz`

### Available tools (7)

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents as text |
| `write_file` | Create or overwrite a file (creates parent dirs) |
| `edit_file` | Find-and-replace first occurrence |
| `create_directory` | Create directory with parents (mkdir -p) |
| `list_directory` | List entries with type indicators (dir suffix `/`) |
| `directory_tree` | Recursive tree with visual connectors |
| `search_files` | Recursive regex search with file:line:match output |

### Health check

`/healthz` endpoint verifies `/workspace` directory is accessible (not just that the server process is running).

### K8s deployment

Unchanged from original — same Helm chart, same service name, same port 3000.

- Service name: `fm-mcp-filesystem`
- URL: `http://fm-mcp-filesystem.<namespace>.svc.cluster.local:3000/mcp`
- Helm chart: `crates/fm-mcp-filesystem/helm/`

### Docker image

Multi-stage Rust build, minimal runtime image (~30MB vs ~250MB):

```dockerfile
FROM rust:1.93-slim-bookworm AS builder
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p fm-mcp-filesystem

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/fm-mcp-filesystem /app/fm-mcp-filesystem
ENTRYPOINT ["/app/fm-mcp-filesystem"]
```

## Consequences

### Positive

- **No transport translation** — rmcp server SDK speaks the same protocol as rmcp client
- **Multi-threaded** — handles concurrent agent connections properly
- **Reliable health checks** — `/healthz` verifies actual filesystem access
- **Consistent tech stack** — Rust, same patterns as all other MCP servers
- **Small image** — ~30MB vs ~250MB
- **No external dependencies** — no Node.js, no npm, no supergateway

### Negative

- Must implement and maintain filesystem tools (7 tools, ~200 lines each)
- Must implement path sandboxing (security-critical, but straightforward)

## Related

- [ADR-0004: MCP for Tool Integration](0004-mcp-for-tool-integration.md)
- [ADR-0006: MCP Client SDK Selection](0006-mcp-client-sdk-selection.md)
- Source: `crates/fm-mcp-filesystem/`
- Pattern reference: `crates/fm-mcp-devtools/`
- [rmcp SDK](https://docs.rs/rmcp/latest/rmcp/)
