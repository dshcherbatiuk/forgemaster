# ADR-0007: Per-Agent-Type Prompt Guidelines

**Status:** Accepted

**Date:** 2026-02-08

**Decision Makers:** CSM-101

**Technical Area:** Agent Prompting, Controller

## Context

When the orchestrator agent creates child agents via the `create_agent` MCP tool, it composes the entire `task_prompt` itself. The orchestrator has only a brief 3-line description of each agent type's role (in `orchestrator-role.md` lines 15-18), and the LLM is free to include or omit any guidelines it sees fit.

This leads to inconsistent agent behavior:
- The orchestrator may forget to include important coding standards or review criteria
- Different runs produce different prompt quality for the same agent type
- There is no way to enforce project-wide rules (e.g., "always use Gherkin Given-When-Then", "always initialize git in workspace") without relying on the LLM to relay them

Critically, the orchestrator dynamically decides which agent types to create based on the task. Agent types are not fixed — the orchestrator may create agents beyond the predefined set (code-generator, test-generator, reviewer) depending on the task domain.

## Decision Drivers

- **Flexibility** — orchestrator must be free to create any agent types the task requires
- **Consistency** — agents should follow baseline quality standards
- **Single source of truth** — all agent guidelines live in one place
- **Simplicity** — minimal code changes, no controller coupling to agent types
- **Maintainability** — easy to update guidelines without changing Rust code

## Considered Options

### Option 1: Controller-side injection (per-agent-type prompt files)

Create separate markdown files per agent type in `resources/prompts/`. The controller prepends the matching role file to the `task_prompt` based on `agent_type` in `CreateAgentAction::execute()`.

**Pros:**
- Guidelines are deterministic and version-controlled
- Consistent baseline per known agent type

**Cons:**
- Controller must know about agent types (tight coupling)
- Unknown/dynamic agent types get no guidelines (breaks flexibility)
- Prompt files compiled into binary (rebuild to update)
- Violates open-closed principle — new agent type requires code change

### Option 2: Expand orchestrator prompt (LLM-side)

Add detailed per-agent-type guidelines directly in `orchestrator-role.md`. The orchestrator receives all rules upfront and is instructed to include the relevant ones when composing each child agent's `task_prompt`.

**Pros:**
- No code changes needed — only edit `orchestrator-role.md`
- Single file to maintain
- Flexible — orchestrator adapts guidelines to any agent type
- No coupling between controller and agent types
- Orchestrator can create new agent types without controller changes

**Cons:**
- Relies on LLM faithfully relaying guidelines
- Longer orchestrator prompt increases token usage
- LLM may summarize or omit some rules

### Option 3: ConfigMap-based prompt templates

Store prompt templates in Kubernetes ConfigMaps. Load at runtime.

**Pros:**
- Hot-reloadable without rebuild

**Cons:**
- Over-engineered for current stage
- Added K8s complexity
- Still requires controller to know agent types

## Decision

**Use Option 2: Expand the orchestrator prompt with per-agent-type guidelines.**

The orchestrator dynamically decides which agents to create based on the task. Hardcoding agent types in the controller (Option 1) would couple it to a fixed set and break when the orchestrator invents new agent types for novel tasks.

Instead, all guidelines live in `orchestrator-role.md`. The existing "Agent types and their responsibilities" section is expanded with detailed rules for each type. The orchestrator is instructed to **always include the relevant guidelines verbatim** in each child agent's `task_prompt`.

This keeps the controller generic (it just passes `task_prompt` through) and the orchestrator fully autonomous in deciding agent composition.

## Implementation

Expand `crates/fm-controller-agent/resources/prompts/orchestrator-role.md` with detailed guidelines per agent type. Structure:

```markdown
Agent types and their guidelines:

### code-generator
- Role: Produces implementation code and unit tests
- Rules:
  - Initialize git repository in workspace
  - Commit after each logical change
  - Write unit tests alongside implementation
  - Follow language-specific conventions
  ...

### test-generator
- Role: Produces E2E acceptance tests in Gherkin format
- Rules:
  - Use Given-When-Then format
  - Cover happy path and error cases
  ...

### reviewer
- Role: Reviews code for correctness and completeness
- Rules:
  - Check against acceptance criteria
  - Verify test coverage
  ...

IMPORTANT: When creating an agent, always include the full guidelines
for its type in the task_prompt.
```

The orchestrator is free to create agents beyond these types — for unknown types it composes guidelines based on its own analysis.

## Consequences

### Positive

- Orchestrator remains free to create any agent types
- Single file contains all guidelines (easy to find and update)
- No code changes needed — only markdown updates
- No coupling between controller and agent types
- Guidelines evolve alongside the orchestrator prompt

### Negative

- LLM may not relay all guidelines perfectly every time
- Orchestrator prompt grows longer (higher token cost per orchestrator call)
- Guidelines are suggestions, not enforced guarantees

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LLM omits guidelines from child prompt | Medium | Medium | Use explicit instruction: "include guidelines verbatim" |
| Orchestrator prompt exceeds context window | Low | High | Keep guidelines concise, monitor token usage |
| Guidelines conflict with task-specific context | Low | Low | Guidelines are general rules, task context is specific |

## Related

- [ADR-0004: MCP for Tool Integration](0004-mcp-for-tool-integration.md)
- [ADR-0006: MCP Client SDK Selection](0006-mcp-client-sdk-selection.md)
- Orchestrator prompt: `crates/fm-controller-agent/resources/prompts/orchestrator-role.md`
- CreateAgentAction: `crates/fm-controller-agent/src/mcp_server/action/create_agent.rs`
