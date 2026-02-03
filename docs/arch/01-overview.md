# Meta-Agent System Architecture

> Autonomous meta-agent: analyzes tasks, creates/selects agents & MCP servers, orchestrates execution

## Overview

A self-regulating meta-agent system inspired by PID controller principles. The system dynamically provisions and orchestrates task-specific AI agents and MCP servers, using E2E tests as the setpoint for measuring success.

## Core Concepts

### TCP Controller (Task-Context-Prediction)

| Component | Term | Function |
|-----------|------|----------|
| Task Analyzer | **T** (Task) | Reacts to current task requirements and error state |
| Context Memory | **C** (Context) | Accumulated history, learned patterns, what worked before |
| Predictor | **P** (Prediction) | Anticipates failures, adapts to rate of change |

### Setpoint: E2E Test Generation (Gherkin)

Instead of hardcoded success criteria, a dedicated **Test Generator Agent** analyzes each incoming task and produces E2E tests in **Gherkin format** (Given-When-Then) that define what success looks like. This makes the setpoint:

- **Dynamic** — generated per task
- **Measurable** — pass/fail, objective
- **Self-documenting** — Gherkin is human-readable
- **Executable** — runs with Cucumber, Behave, etc.

### Error Signal

```
error = failed_tests / total_tests
```

The error signal drives the feedback loop, triggering adjustments in agent selection, configuration, or strategy.

---
