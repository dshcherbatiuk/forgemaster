# Context Management

## Overview

**Context** is the shared state that flows between agents during task execution. It includes task requirements, generated artifacts, test results, and iteration history.

## Why External Context Storage Matters

Traditional LLM agents lose information when context gets compressed:

```
LLM Context Problem:
────────────────────
Conversation grows → Context window fills → Compress/summarize → Details LOST
```

ForgeMaster solves this by storing context **externally**:

```
ForgeMaster Solution:
─────────────────────
Context stored in Redis/DB → LLM queries what it needs → Nothing LOST
```

**Key Insight:** The TCP Controller uses **math, not LLM** for orchestration decisions. LLM agents can "forget" details after compression, but the Context Store preserves everything. When an agent needs historical context:

1. Query Context Store for relevant data
2. Inject into agent prompt
3. Agent works with full context
4. Results saved back to Context Store

This means ForgeMaster maintains **perfect memory** across iterations, tasks, and agent lifetimes.

## LLM Integration Pattern

How to integrate Claude API with external context storage:

```mermaid
sequenceDiagram
    participant Agent as LLM Agent
    participant Context as Context Store (Redis)
    participant Claude as Claude API

    Note over Agent: Task: "Fix the auth bug"

    Agent->>Context: Query relevant context
    Context-->>Agent: Previous iterations, error history, artifacts

    Agent->>Agent: Build prompt with injected context

    Agent->>Claude: API call (prompt + context)
    Claude-->>Agent: Response

    Agent->>Context: Save new results, decisions, artifacts
```

### Smart Context Selection

Don't inject ALL context (too large). Select what's relevant.

### What Gets Stored vs Injected

| Stored (Everything) | Injected (Relevant Only) |
|---------------------|--------------------------|
| All iterations | Last 3-5 iterations |
| All artifacts | Current artifacts + diffs |
| All decisions | Recent decisions |
| Full test history | Failing tests only |
| All agent outputs | Summaries |

> **Key Principle:** LLM sees a "window" into the context store. The store has everything. The LLM gets what's relevant for THIS call.

## Context Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CONTEXT MANAGEMENT                                │
│                                                                          │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │
│   │   Agent 1   │    │   Agent 2   │    │   Agent 3   │                 │
│   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                 │
│          │                  │                  │                         │
│          │    read/write    │    read/write    │                         │
│          └──────────────────┼──────────────────┘                         │
│                             │                                            │
│                             ▼                                            │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                     CONTEXT STORE                                │   │
│   │                        (Redis)                                   │   │
│   │                                                                  │   │
│   │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │   │
│   │   │    Task     │  │  Artifacts  │  │   History   │             │   │
│   │   │   Context   │  │   Store     │  │    Log      │             │   │
│   │   └─────────────┘  └─────────────┘  └─────────────┘             │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                             │                                            │
│                             ▼                                            │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                   PERSISTENT STORAGE                             │   │
│   │                      (S3/GitHub)                                 │   │
│   │                                                                  │   │
│   │   Checkpoints, Final Artifacts, Archived Context                 │   │
│   └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Context Schema

### Task Context

```json
{
  "$schema": "https://forgemaster.io/schemas/context/v1",
  "task_id": "a1b2c3d4",
  "version": 5,
  "created_at": "2026-02-04T10:00:00Z",
  "updated_at": "2026-02-04T10:25:00Z",

  "task": {
    "name": "ecommerce-backend",
    "description": "Create e-commerce backend with product catalog, shopping cart, and checkout",
    "requirements": {
      "type": "web-api",
      "language": "rust",
      "framework": "axum",
      "features": ["product-catalog", "shopping-cart", "checkout", "stripe-integration"]
    },
    "acceptance_criteria": [
      "Given product exists, when POST /cart/items, then add to cart",
      "Given items in cart, when POST /checkout, then process payment via Stripe"
    ]
  },

  "execution": {
    "iteration": 3,
    "status": "running",
    "current_agent": "code-generator-abc123",
    "started_at": "2026-02-04T10:00:00Z"
  },

  "agents": {
    "deployed": [
      {
        "id": "test-generator-xyz789",
        "type": "test-generator",
        "status": "completed"
      },
      {
        "id": "code-generator-abc123",
        "type": "code-generator",
        "status": "running"
      }
    ]
  },

  "artifacts": {
    "tests": {
      "path": "artifacts/tests/ecommerce-api.feature",
      "checksum": "sha256:abc123...",
      "created_by": "test-generator-xyz789",
      "iteration": 1
    },
    "code": {
      "path": "artifacts/src/",
      "files": ["main.rs", "cart.rs", "checkout.rs", "stripe.rs", "models.rs"],
      "checksum": "sha256:def456...",
      "created_by": "code-generator-abc123",
      "iteration": 3
    }
  },

  "test_results": {
    "latest": {
      "iteration": 3,
      "passed": 6,
      "failed": 2,
      "total": 8,
      "error_signal": 0.25,
      "failures": [
        {
          "test": "checkout-payment-validation",
          "scenario": "Given invalid payment, when checkout, then reject",
          "error": "Expected 402, got 500",
          "file": "src/checkout.rs",
          "line": 78
        }
      ]
    },
    "history": [
      {"iteration": 1, "passed": 2, "failed": 6, "error_signal": 0.75},
      {"iteration": 2, "passed": 4, "failed": 4, "error_signal": 0.50},
      {"iteration": 3, "passed": 6, "failed": 2, "error_signal": 0.25}
    ]
  },

  "decisions": [
    {
      "iteration": 1,
      "tcp_signal": 0.75,
      "action": "ADD_AGENT",
      "details": "Added payment-specialist agent for Stripe integration"
    },
    {
      "iteration": 2,
      "tcp_signal": 0.50,
      "action": "ADJUST_PARAMS",
      "details": "Updated code-generator prompt with Stripe error handling examples"
    }
  ],

  "metadata": {
    "subsystem_template": "ecommerce-backend",
    "tcp_coefficients": {"task": 1.0, "context": 0.5, "prediction": 0.3},
    "resource_usage": {
      "llm_calls": 45,
      "tokens_used": 125000,
      "estimated_cost": 1.25
    }
  }
}
```

## Context Flow Between Agents

### Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     CONTEXT FLOW                                         │
│                                                                          │
│  1. Test Generator                                                       │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │  READ:  task.requirements, task.acceptance_criteria         │     │
│     │  WRITE: artifacts.tests                                      │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  2. Code Generator                                                       │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │  READ:  task.requirements, artifacts.tests,                 │     │
│     │         test_results.failures (if retry)                     │     │
│     │  WRITE: artifacts.code                                       │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  3. Reviewer                                                             │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │  READ:  artifacts.code, artifacts.tests, task.requirements   │     │
│     │  WRITE: artifacts.review                                      │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  4. Test Runner                                                          │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │  READ:  artifacts.tests, artifacts.code                      │     │
│     │  WRITE: test_results.latest, test_results.history            │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  5. TCP Controller                                                       │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │  READ:  test_results.latest.error_signal,                   │     │
│     │         test_results.history                                 │     │
│     │  WRITE: decisions (via Orchestrator)                         │     │
│     └─────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
```

## Context Storage

### Redis Schema

```
# Task context (main state)
task:{task_id}:context → JSON (TaskContext)

# Artifacts (large files stored as references)
task:{task_id}:artifacts:{type} → JSON (ArtifactMetadata)
task:{task_id}:artifacts:{type}:content → BLOB (if small) or S3 reference

# History log (append-only)
task:{task_id}:history → LIST of JSON entries

# Agent states
task:{task_id}:agents:{agent_id} → JSON (AgentState)

# Pub/sub channels
task:{task_id}:context:updated → version number
task:{task_id}:agents:status → agent status changes

# TTL for cleanup
task:{task_id}:* → TTL 24h after task completion
```

### Large Artifact Handling

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   ARTIFACT STORAGE STRATEGY                              │
│                                                                          │
│   Artifact Size        Storage Location                                  │
│   ─────────────────────────────────────────────                          │
│   < 1 KB              Redis (inline in context)                          │
│   1 KB - 1 MB         Redis (separate key)                               │
│   > 1 MB              S3/GitHub (reference in Redis)                     │
│                                                                          │
│   Example:                                                               │
│   {                                                                      │
│     "artifacts": {                                                       │
│       "tests": {                                                         │
│         "storage": "inline",                                             │
│         "content": "Feature: User API..."                                │
│       },                                                                 │
│       "code": {                                                          │
│         "storage": "s3",                                                 │
│         "bucket": "forgemaster-artifacts",                               │
│         "key": "task-a1b2c3d4/code/v3.tar.gz",                          │
│         "checksum": "sha256:..."                                         │
│       }                                                                  │
│     }                                                                    │
│   }                                                                      │
└─────────────────────────────────────────────────────────────────────────┘
```

## Context Checkpoints

### Checkpoint Strategy

```yaml
checkpoints:
  # When to create checkpoints
  triggers:
    - after_each_iteration
    - before_agent_swap
    - on_significant_progress  # error drops by > 0.2

  # What to include
  contents:
    - full context snapshot
    - all artifact references
    - agent configurations

  # Where to store
  storage:
    location: s3://forgemaster-checkpoints/
    path: "{task_id}/checkpoint-{iteration}.json"
    retention: 7 days

  # Recovery
  recovery:
    - On agent failure: restore from last checkpoint
    - On task resume: load checkpoint, continue
```

### Checkpoint Format

```json
{
  "checkpoint_id": "task-a1b2c3d4-iter-3",
  "created_at": "2026-02-04T10:20:00Z",
  "task_id": "a1b2c3d4",
  "iteration": 3,

  "context_snapshot": {
    "...": "full TaskContext JSON"
  },

  "agent_states": {
    "code-generator-abc123": {
      "status": "completed",
      "last_output": "s3://artifacts/code-gen-output-3.json"
    }
  },

  "recovery_instructions": {
    "resume_from": "code-generator-abc123",
    "next_step": "run_tests",
    "pending_agents": ["reviewer"]
  }
}
```

## Context API

### REST Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/tasks/{id}/context` | Get full context |
| GET | `/api/v1/tasks/{id}/context/artifacts` | List artifacts |
| GET | `/api/v1/tasks/{id}/context/artifacts/{type}` | Get artifact |
| PUT | `/api/v1/tasks/{id}/context/artifacts/{type}` | Store artifact |
| GET | `/api/v1/tasks/{id}/context/history` | Get history |
| GET | `/api/v1/tasks/{id}/context/test-results` | Get test results |
| POST | `/api/v1/tasks/{id}/context/checkpoint` | Create checkpoint |
| GET | `/api/v1/tasks/{id}/context/checkpoints` | List checkpoints |

### WebSocket for Real-time Updates

```javascript
// Client subscribes to context updates
ws.connect(`wss://api.forgemaster.io/tasks/${taskId}/context/stream`);

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);

  switch (update.type) {
    case 'artifact_added':
      console.log(`New artifact: ${update.artifact_type}`);
      break;
    case 'test_results':
      console.log(`Tests: ${update.passed}/${update.total} passed`);
      break;
    case 'iteration_complete':
      console.log(`Iteration ${update.iteration} complete`);
      break;
    case 'decision':
      console.log(`Decision: ${update.action}`);
      break;
  }
};
```

## Context Lifecycle

### Creation

```
1. Task submitted
2. Create initial context with:
   - Task requirements
   - Empty artifacts
   - Empty history
   - Initial execution state
3. Store in Redis
4. Create initial checkpoint
```

### During Execution

```
1. Agents read/write context
2. Context version increments on each write
3. Pub/sub notifies interested parties
4. Checkpoints created at iteration boundaries
```

### Completion

```
1. Task completes (success or failure)
2. Create final checkpoint
3. Archive context to S3
4. Start TTL countdown in Redis
5. After TTL: Remove from Redis, keep S3 archive
```

### Cleanup

```yaml
cleanup_policy:
  redis:
    success_tasks: 1 hour after completion
    failed_tasks: 24 hours after completion

  s3_archives:
    success_tasks: 90 days
    failed_tasks: 30 days

  checkpoints:
    during_execution: keep all
    after_completion: keep final only
```

## Context Security

### Access Control

```yaml
access_control:
  # Agents can only access their task's context
  agent_scope: task_namespace

  # Read permissions
  read:
    - orchestrator: full context
    - agents: task context, own artifacts
    - tcp_controller: test_results, history

  # Write permissions
  write:
    - orchestrator: full context
    - agents: own artifacts, own status
    - tcp_controller: none (reads only)
    - test_runner: test_results
```

### Data Sanitization

```
Before passing context to agent:
1. Remove sensitive fields (API keys, secrets)
2. Truncate large artifacts (provide references)
3. Summarize history if too long
4. Apply agent-specific view filters
```
