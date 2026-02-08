# ADR-0010: Anthropic API Mock Server for Testing

**Status:** Proposed

**Date:** 2026-02-08

**Decision Makers:** CSM-101

**Technical Area:** Testing, Claude API Integration

## Context

The agent runtime (`fm-agent-runtime-claude`) sends streaming SSE requests to the Anthropic Claude Messages API (`/v1/messages`). Currently there are no integration tests for the conversation loop because there is no way to test against the real API in CI (costs money, non-deterministic, rate-limited).

We need a mock server that:
- Accepts POST requests to `/v1/messages` with the Anthropic request format
- Returns real incremental SSE events in the Anthropic format (`message_start` → `content_block_start` → `content_block_delta` → `content_block_stop` → `message_delta` → `message_stop`)
- Supports configurable responses (text, tool_use, errors, stop reasons)
- Runs in-process alongside Rust tests (no external services)

## Decision Drivers

- **SSE streaming fidelity** — must produce real incremental SSE, not buffered responses
- **Native Rust** — no JVM or external process dependencies
- **Test control** — configurable responses, error injection, timing control
- **Zero CI cost** — runs locally without API keys
- **Minimal dependencies** — reuse existing crates (Axum, Tokio already in the project)

## Considered Options

### Option 1: Custom Axum Mock Server

Build a lightweight mock `/v1/messages` endpoint using Axum's built-in SSE support (`axum::response::sse::Sse`). Runs as a tokio task in tests.

**Pros:**
- Native Rust, same async runtime (Tokio) as the project
- Real incremental SSE streaming via `axum::response::sse`
- Full control over event sequence, timing, error scenarios
- Runs in-process — no external services, no Docker
- Axum already in project dependencies
- Can test edge cases: `max_tokens` truncation, `tool_use` responses, API errors, rate limits

**Cons:**
- Must build and maintain the mock implementation
- Must keep mock events in sync with Anthropic API format changes

### Option 2: Mokksy / AI-Mocks (Kotlin/JVM)

[Mokksy](https://mokksy.dev/docs/ai-mocks/anthropic/) provides a full mock server for the Anthropic API with SSE streaming, configurable via Kotlin DSL.

**Pros:**
- Full Anthropic API mock with real SSE streaming
- Configurable responses, delays, matching conditions
- Battle-tested for LLM API testing

**Cons:**
- Requires JVM 17+ runtime — heavyweight for a Rust project
- No published Docker image — must build from source with Gradle
- Configuration via Kotlin DSL, not Rust
- Adds JVM to CI pipeline

### Option 3: wiremock-rs / httpmock / mockito

Standard Rust HTTP mocking libraries.

**Pros:**
- Pure Rust, easy to use
- Good for request/response matching

**Cons:**
- No SSE streaming support — responses are static, fully buffered
- Cannot test incremental event processing
- Defeats the purpose of testing the streaming conversation loop

### Option 4: No Mock — Test Against Real API

Use the real Anthropic API with a test API key.

**Pros:**
- Tests against the real implementation
- No mock maintenance

**Cons:**
- Costs money per test run
- Non-deterministic responses — flaky tests
- Rate-limited — CI bottleneck
- Requires API key in CI secrets
- Cannot test error scenarios (rate limits, server errors) on demand

## Decision

**Use Option 1: Custom Axum Mock Server.**

Axum's `axum::response::sse::Sse` produces real incremental SSE events. The mock server runs as a tokio task alongside tests — no external processes, no JVM, no Docker. The project already depends on Axum and Tokio.

The mock server will live in a `test_support` module within `fm-agent-runtime-claude` and provide:
- Configurable text responses with `end_turn` stop reason
- Configurable `tool_use` responses for testing the tool calling loop
- `max_tokens` truncation responses for testing continuation
- API error responses (rate limit, authentication, server error)
- Proper Anthropic SSE event sequence

## Consequences

### Positive

- Integration tests for the full conversation loop (currently untested)
- Tests for edge cases: `max_tokens` continuation, tool calling cycles, API errors
- Zero CI cost — no API keys needed
- Deterministic — same response every time
- Fast — no network latency to real API

### Negative

- Must maintain the mock when Anthropic changes their SSE format
- Mock may diverge from real API behaviour over time

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Mock diverges from real API format | Low | Medium | Keep mock event format aligned with `claude_api::sse` parsers — if parsers change, mock must change |
| Mock doesn't cover all API edge cases | Medium | Low | Add scenarios incrementally as bugs are found in production |
| Axum SSE API changes | Low | Low | Pin Axum version, update on major upgrades |

## Related

- [Axum SSE module](https://docs.rs/axum/latest/axum/response/sse/index.html)
- [Mokksy AI-Mocks Anthropic](https://mokksy.dev/docs/ai-mocks/anthropic/)
- [wiremock-rs](https://github.com/LukeMathWalker/wiremock-rs)
- [httpmock](https://github.com/alexliesenfeld/httpmock)
