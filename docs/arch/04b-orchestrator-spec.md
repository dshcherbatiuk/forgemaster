# Orchestrator Agent Specification

## Overview

The **Orchestrator Agent** is the brain of ForgeMaster. It receives tasks, creates agent subnetworks, assigns work, and interprets TCP Controller signals to adjust execution.

## Responsibilities

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      ORCHESTRATOR AGENT                                  │
│                                                                          │
│   1. TASK ANALYSIS                                                       │
│      - Parse task requirements                                           │
│      - Determine required skills                                         │
│      - Estimate complexity and agents needed                             │
│                                                                          │
│   2. AGENT PROVISIONING                                                  │
│      - Query Agent Registry for existing agents                          │
│      - Create new agents when needed                                     │
│      - Configure MCP servers                                             │
│                                                                          │
│   3. WORK ASSIGNMENT                                                     │
│      - Break task into subtasks                                          │
│      - Assign subtasks to agents                                         │
│      - Coordinate agent communication                                    │
│                                                                          │
│   4. SIGNAL INTERPRETATION                                               │
│      - Receive TCP Controller signals                                    │
│      - Translate signals to concrete actions                             │
│      - Execute actions (swap, add, adjust)                               │
│                                                                          │
│   5. CONTEXT MANAGEMENT                                                  │
│      - Maintain task context                                             │
│      - Pass context between agents                                       │
│      - Consolidate feedback                                              │
└─────────────────────────────────────────────────────────────────────────┘
```

## Architecture

```
                    ┌─────────────────────┐
                    │    Task Manager     │
                    │  (receives tasks)   │
                    └──────────┬──────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        ORCHESTRATOR AGENT                                │
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                   │
│  │    Task      │  │    Agent     │  │    Work      │                   │
│  │   Analyzer   │  │  Provisioner │  │  Assigner    │                   │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                   │
│         │                 │                 │                            │
│         └─────────────────┴─────────────────┘                            │
│                           │                                              │
│                           ▼                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    DECISION ENGINE                                │   │
│  │                                                                   │   │
│  │   TCP Signal ───▶ Action Mapper ───▶ Executor                    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
       ┌────────────┐   ┌────────────┐   ┌────────────┐
       │   Agent    │   │   Agent    │   │   Agent    │
       │  Registry  │   │    Pool    │   │   Context  │
       └────────────┘   └────────────┘   └────────────┘
```

## Orchestrator CRD

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: orchestrator
  namespace: forgemaster-system
spec:
  model:
    provider: anthropic
    name: claude-3-opus  # Strongest model for orchestration
    temperature: 0.2
    max_tokens: 8192

  system_prompt: |
    You are the Orchestrator Agent for ForgeMaster.

    Your role:
    1. Analyze tasks and determine required agents
    2. Create agent configurations when needed
    3. Assign work to agents
    4. Interpret control signals and take corrective action

    You have access to:
    - Agent Registry API (query, create, update agents)
    - Task Context Store (read/write task state)
    - MCP Server Manager (provision tools)

    Decision framework:
    - Prefer reusing existing agents over creating new
    - Start with minimal agents, add only when needed
    - When error signal is high, try different approach
    - Document all decisions for learning

  skills:
    - name: orchestration
      proficiency: 1.0
    - name: agent-management
      proficiency: 1.0
    - name: task-analysis
      proficiency: 0.95

  mcp_servers:
    - name: agent-registry-mcp
    - name: context-store-mcp
    - name: k8s-mcp

  singleton: true  # Only one orchestrator per cluster
```

## Decision Engine

### TCP Signal to Action Mapping

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     TCP SIGNAL INTERPRETATION                            │
│                                                                          │
│   Signal Range        │  Action              │  Orchestrator Behavior    │
│   ────────────────────┼──────────────────────┼────────────────────────── │
│   signal > 0.7        │  SWAP_AGENT          │  Replace failing agent    │
│                       │                      │  with different type      │
│   ────────────────────┼──────────────────────┼────────────────────────── │
│   0.5 < signal <= 0.7 │  ADD_AGENT           │  Add specialist agent     │
│                       │                      │  to assist                │
│   ────────────────────┼──────────────────────┼────────────────────────── │
│   0.2 < signal <= 0.5 │  ADJUST_PARAMS       │  Modify agent prompts     │
│                       │                      │  or context               │
│   ────────────────────┼──────────────────────┼────────────────────────── │
│   signal <= 0.2       │  CONTINUE            │  Keep current approach    │
│                       │                      │  progress is good         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Action Execution

#### SWAP_AGENT

```yaml
trigger: signal > 0.7
analysis:
  - Identify underperforming agent
  - Analyze failure patterns from test results
  - Determine better agent type

actions:
  1. Query registry for alternative agents with:
     - Different skill combination
     - Higher proficiency in failing area
     - Different model (e.g., opus instead of sonnet)

  2. If no suitable agent exists:
     - Generate new agent config with adjusted approach
     - Deploy new agent

  3. Migrate context from old to new agent

  4. Terminate old agent

  5. Update task state

example:
  before:
    agent: code-generator-sonnet
    failing_tests: ["authentication tests"]

  after:
    agent: security-specialist-opus
    reasoning: "Auth tests failing, need security expertise"
```

#### ADD_AGENT

```yaml
trigger: 0.5 < signal <= 0.7
analysis:
  - Current agents insufficient but not failing
  - Specific skill gap identified
  - Additional expertise needed

actions:
  1. Analyze which tests are failing

  2. Map failures to missing skills

  3. Query or create specialist agent:
     - "validation-specialist" for validation tests
     - "security-reviewer" for security tests
     - "performance-optimizer" for performance tests

  4. Configure new agent with context from current state

  5. Add to agent pool without removing existing

example:
  current_agents: [test-generator, code-generator]
  failing_tests: ["input validation", "error handling"]
  action: Add "validation-specialist" agent
```

#### ADJUST_PARAMS

```yaml
trigger: 0.2 < signal <= 0.5
analysis:
  - Progress being made but slow
  - Minor issues, not fundamental problems
  - Tuning might help

actions:
  1. Analyze recent iterations for patterns

  2. Adjust agent parameters:
     - Modify system prompts with more context
     - Change temperature (lower for more consistency)
     - Add examples of expected output
     - Increase context window

  3. Update agent configuration

  4. Continue with same agents

example:
  observation: "Code compiles but tests fail on edge cases"
  adjustment:
    - Add edge case examples to code-generator prompt
    - Lower temperature from 0.3 to 0.1
```

#### CONTINUE

```yaml
trigger: signal <= 0.2
analysis:
  - Good progress
  - No changes needed

actions:
  1. Log positive progress
  2. Maintain current configuration
  3. Proceed to next iteration
```

## Work Assignment Algorithm

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      WORK ASSIGNMENT                                     │
│                                                                          │
│  Input: Task + Available Agents                                          │
│                                                                          │
│  Step 1: Break down task                                                 │
│    task: "Create e-commerce backend with catalog, cart, checkout"        │
│    subtasks:                                                             │
│      - Generate Gherkin tests for e-commerce flows                       │
│      - Implement catalog, cart, checkout endpoints                       │
│      - Integrate Stripe payment processing                               │
│      - Review code for issues                                            │
│      - Run tests and collect feedback                                    │
│                                                                          │
│  Step 2: Match subtasks to agents                                        │
│    subtask                    │  assigned_agent                          │
│    ──────────────────────────┼─────────────────                          │
│    Generate tests            │  test-generator                           │
│    Implement endpoints       │  code-generator                           │
│    Review code               │  reviewer                                 │
│    Collect feedback          │  feedback-analyzer                        │
│                                                                          │
│  Step 3: Define execution order                                          │
│    1. test-generator (parallel start)                                    │
│    2. code-generator (after tests ready)                                 │
│    3. reviewer (after code ready)                                        │
│    4. feedback-analyzer (after tests run)                                │
│                                                                          │
│  Step 4: Assign via A2A protocol                                         │
│    Send task messages to each agent with context                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Context Passing

### Context Structure

```json
{
  "task_id": "a1b2c3d4",
  "iteration": 3,
  "requirements": {
    "type": "web-api",
    "language": "rust",
    "features": ["catalog", "cart", "checkout", "stripe"]
  },
  "artifacts": {
    "tests": "path/to/tests.feature",
    "code": "path/to/src/",
    "reviews": ["review-1.md", "review-2.md"]
  },
  "history": [
    {
      "iteration": 1,
      "agent": "code-generator",
      "action": "generate",
      "result": "partial success",
      "failing_tests": ["auth-test-1", "auth-test-2"]
    },
    {
      "iteration": 2,
      "agent": "code-generator",
      "action": "fix",
      "result": "improved",
      "failing_tests": ["auth-test-2"]
    }
  ],
  "current_error": 0.25,
  "test_results": {
    "passed": 6,
    "failed": 2,
    "total": 8,
    "failures": [
      {
        "test": "auth-test-2",
        "reason": "Token validation fails for expired tokens"
      }
    ]
  }
}
```

### Context Flow

```
┌─────────────┐    context    ┌─────────────┐    context    ┌─────────────┐
│    Test     │──────────────▶│    Code     │──────────────▶│  Reviewer   │
│  Generator  │               │  Generator  │               │             │
└─────────────┘               └─────────────┘               └──────┬──────┘
      │                             │                              │
      │                             │                              │
      ▼                             ▼                              ▼
┌───────────────────────────────────────────────────────────────────────┐
│                        CONTEXT STORE (Redis)                          │
│                                                                        │
│  task:a1b2c3d4:context = {...}                                        │
│  task:a1b2c3d4:artifacts:tests = "..."                                │
│  task:a1b2c3d4:artifacts:code = "..."                                 │
│  task:a1b2c3d4:history = [...]                                        │
└───────────────────────────────────────────────────────────────────────┘
```

## Error Handling

### Agent Failure

```yaml
scenario: Agent pod crashes or becomes unresponsive

detection:
  - Heartbeat timeout (30 seconds)
  - Health check failure
  - Pod termination event

response:
  1. Mark agent as FAILED in registry
  2. Retrieve context from last checkpoint
  3. Provision replacement agent (same config)
  4. Resume from checkpoint
  5. Log failure for pattern analysis
```

### Repeated Failures

```yaml
scenario: Same agent type fails multiple times

detection:
  - 3+ failures of same agent type
  - Error patterns indicate fundamental issue

response:
  1. Escalate to SWAP_AGENT action
  2. Try different agent configuration:
     - Different model
     - Different prompt strategy
     - Additional MCP servers
  3. If still failing after 3 swaps:
     - Mark task as requiring human review
     - Provide detailed failure report
```

### Resource Exhaustion

```yaml
scenario: Namespace hits resource quota

detection:
  - Pod pending due to quota
  - OOM kills

response:
  1. Identify lowest-priority agents
  2. Scale down or terminate idle agents
  3. Request quota increase if possible
  4. If critical, pause task and alert
```

## Metrics and Observability

### Orchestrator Metrics

```
forgemaster_orchestrator_tasks_total{status}
forgemaster_orchestrator_agents_created_total
forgemaster_orchestrator_decisions_total{action}
forgemaster_orchestrator_decision_latency_seconds
forgemaster_orchestrator_agent_swaps_total{reason}
forgemaster_orchestrator_context_size_bytes
```

### Decision Logging

```json
{
  "timestamp": "2026-02-04T10:15:30Z",
  "task_id": "a1b2c3d4",
  "iteration": 3,
  "tcp_signal": 0.55,
  "decision": "ADD_AGENT",
  "reasoning": "Validation tests failing, current agents lack validation expertise",
  "action_taken": {
    "type": "create_agent",
    "agent_type": "validation-specialist",
    "agent_id": "validation-spec-xyz789"
  },
  "context_snapshot": "s3://artifacts/task-a1b2c3d4/context-iter-3.json"
}
```
