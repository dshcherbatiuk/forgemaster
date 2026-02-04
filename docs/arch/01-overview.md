# Meta-Agent System Architecture

> Autonomous meta-agent: analyzes tasks, creates/selects agents & MCP servers, orchestrates execution

## Overview

A self-regulating meta-agent system using TCP (Task-Context-Prediction) control. The system dynamically provisions and orchestrates task-specific AI agents and MCP servers, using dual validation (tests + outcomes) as the setpoint for measuring success.

## Core Concepts

### TCP Controller (Task-Context-Prediction)

| Component | Term | Function |
|-----------|------|----------|
| Task Analyzer | **T** (Task) | Reacts to current task requirements and error state |
| Context Memory | **C** (Context) | Accumulated history, learned patterns, what worked before |
| Predictor | **P** (Prediction) | Anticipates failures, adapts to rate of change |

### Setpoint: Dual Validation (Tests + Outcomes)

The system uses **dual validation** to ensure autonomous correctness:

1. **Test Validation** — Generated E2E tests (Gherkin) verify implementation
2. **Outcome Validation** — Real-world checks verify the task actually worked

#### Test Validation (Gherkin)

A dedicated **Test Generator Agent** produces E2E tests in **Gherkin format** (Given-When-Then):

- **Dynamic** — generated per task
- **Measurable** — pass/fail, objective
- **Self-documenting** — Gherkin is human-readable
- **Executable** — runs with Cucumber, Behave, etc.

#### Outcome Validation

Since tests are self-generated (system grades its own homework), we add **real-world outcome checks**:

| Task Type | Outcome Verification |
|-----------|---------------------|
| Ticket Booking | Confirmation email received, booking ID valid in external system |
| ETL Pipeline | Data exists in target, row counts match, checksums valid |
| API Development | External client can call endpoints, integration tests pass |
| ML Training | Metrics improve on held-out validation set |

> **Principle:** Tests verify the HOW. Outcomes verify the WHAT.

### Error Signal

```
error = (failed_tests + failed_outcomes) / total_checks
```

The error signal drives the feedback loop, triggering adjustments in agent selection, configuration, or strategy.

---
