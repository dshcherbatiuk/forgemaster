# ADR-0001: TCP (PID) Controller vs LLM for Orchestration

**Status:** Accepted

**Date:** 2026-02-03

**Decision Makers:** Forgemaster Team

**Technical Area:** System Architecture / Orchestration

## Context

The Meta-Agent System requires a controller to orchestrate agent execution. The controller must:
- Measure error signals (failed_tests / total_tests)
- Decide control actions (swap agent, retry, add agent)
- React to feedback from test execution

The key question is: Should the controller use an LLM for decision-making, or a simpler mathematical approach (PID-like control)?

```
┌─────────────────────────────────────────────────────────────────┐
│                    CONTROLLER INPUT                              │
│                                                                  │
│   • error = 0.4 (number)                                        │
│   • previous_error = 0.5 (number)                               │
│   • iteration = 3 (number)                                      │
│                                                                  │
│   Question: "Should I swap agent, add agent, or continue?"      │
│                                                                  │
│   This is a MATHEMATICAL decision, not a CREATIVE one.          │
└─────────────────────────────────────────────────────────────────┘
```

## Decision Drivers

- **Speed**: Controller makes many decisions per task execution
- **Cost**: Budget should be reserved for creative agent work
- **Determinism**: Same input should produce same output for predictability
- **Reliability**: Controller must always work, even if external APIs fail
- **Scalability**: System may need to handle many concurrent tasks
- **Debuggability**: Ability to trace and understand decisions

## Considered Options

### Option 1: PID Controller (Mathematical)

Pure Rust implementation using PID-inspired control logic:

```rust
let signal = Kt * error + Kc * integral + Kp * derivative;

match signal {
    s if s > 0.5 => SwapAgent,    // High error → drastic change
    s if s > 0.2 => AddAgent,     // Medium error → add help
    _ => Continue,                 // Low error → keep going
}
```

**Pros:**
- Speed: 0.001ms per decision (microseconds)
- Cost: $0 per decision
- 100% deterministic (same input → same output)
- Always works (no external dependencies)
- Scalable to 1M+ decisions/sec
- Easy to trace and debug

**Cons:**
- Generic decisions (doesn't understand domain)
- Cannot explain reasoning in natural language
- Requires tuning coefficients (Kt, Kc, Kp)

### Option 2: LLM Controller

Use Claude API for orchestration decisions:

```rust
let prompt = format!(
    "Error is {}. Previous error was {}. Should I swap, add, or continue?",
    error, previous_error
);
let decision = llm.complete(&prompt).await?;
```

**Pros:**
- Can make nuanced, context-aware decisions
- Can explain decisions in natural language
- Adapts to complex scenarios without explicit rules
- More targeted decisions ("add validation-specialist" vs "add agent")

**Cons:**
- Speed: ~2000ms per decision (2M× slower)
- Cost: ~$0.01 per decision
- Non-deterministic (same input may produce different output)
- External API dependency (can fail/timeout)
- Limited scalability (~1 decision/sec)
- Hard to trace and debug

## Decision

**Use PID Controller (Option 1)** for orchestration decisions.

The controller doesn't need to **understand** — it just needs to **react** to numerical signals. This is fundamentally a mathematical decision, not a creative one.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     TCP CONTROLLER (PID)                        │
│                         Rust + Math                             │
│                                                                 │
│   Role: ORCHESTRATION — "What to do"                           │
│   • Measure error signal                                        │
│   • Compute control action (swap, retry, add agent)            │
│   • Fast, deterministic, free                                   │
└─────────────────────────────────┬───────────────────────────────┘
                                  │ Control Signal
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                       AGENTS (LLM)                              │
│                      Claude API                                 │
│                                                                 │
│   Role: EXECUTION — "How to do"                                │
│   • Generate Gherkin tests                                      │
│   • Write code                                                  │
│   • Review and fix                                              │
│   • Creative, context-aware                                     │
└─────────────────────────────────────────────────────────────────┘
```

### Configuration

```yaml
tcp_controller:
  type: pid
  implementation: rust
  latency: "<1ms"
  cost: "$0"
  coefficients:
    task: 1.0
    context: 0.5
    prediction: 0.3

agents:
  type: llm
  implementation: claude-api
  latency: "1-5s"
  cost: "~$0.01/call"
  instances:
    - test_generator    # Creative: writes Gherkin
    - code_generator    # Creative: writes code
    - reviewer          # Creative: finds issues
    - feedback          # Creative: summarizes results
```

## Consequences

### Positive

- **Speed**: Controller decisions in microseconds, not seconds
- **Cost efficiency**: LLM budget reserved for creative agent work
- **Reliability**: Deterministic orchestration with no external dependencies
- **Scalability**: Can handle high-throughput scenarios

### Negative

- **Generic decisions**: Controller cannot make domain-specific choices
- **Tuning required**: PID coefficients need calibration
- **No natural language explanations**: Cannot explain "why" to users

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| PID coefficients poorly tuned | Medium | Medium | Start with defaults, tune based on metrics |
| Edge cases not handled by simple rules | Low | Medium | Agents handle complexity; controller just orchestrates |
| Users want explanation of decisions | Low | Low | Log all decisions; agents can summarize if needed |

## Related

- [01-overview.md](../arch/01-overview.md) — System overview with TCP control model
- [03-feedback-loop.md](../arch/03-feedback-loop.md) — Feedback loop flow and control logic

---

## Appendix: Detailed Analysis

### Comparison by Scenario

**Scenario 1: Error Decreasing (0.6 → 0.4 → 0.2)**

| Controller | Decision | Time | Cost |
|------------|----------|------|------|
| PID | Continue (derivative negative) | 0.001ms | $0 |
| LLM | "Error decreasing, continue" | 2000ms | $0.01 |

→ Same result, PID is 2M× faster and free

**Scenario 2: High Error (0.8)**

| Controller | Decision | Time | Cost |
|------------|----------|------|------|
| PID | Swap agent (error > 0.5) | 0.001ms | $0 |
| LLM | "High error, analyzing why... swap agent" | 2000ms | $0.01 |

→ Same result, PID is faster

**Scenario 3: Stuck Error (0.4 → 0.4 → 0.4)**

| Controller | Decision | Quality |
|------------|----------|---------|
| PID | Swap agent (threshold) | Generic but works |
| LLM | "Same tests failing, add validation-specialist" | More targeted |

→ LLM is smarter, but PID still solves the problem

### Cost Analysis (100 Tasks/Day)

```
Average: 5 iterations per task = 500 control decisions/day

PID Controller:
├── 500 decisions × $0 = $0/day
├── 500 decisions × 0.001ms = 0.5ms total
└── Total: FREE, INSTANT

LLM Controller:
├── 500 decisions × $0.01 = $5/day
├── 500 decisions × 2000ms = 16.7 minutes total
└── Total: $150/month, SLOW

LLM Agents (actual work):
├── ~2000 LLM calls for actual work = $20/day
└── This cost is NECESSARY — agents do real work
```

### Analogy: Factory Floor

```
TCP Controller = Factory Manager
├── Looks at metrics (error rate, throughput)
├── Makes decisions: "Station 3 is slow, add worker"
├── Doesn't need to know HOW to build the product
└── Just optimizes the process

LLM Agents = Skilled Workers
├── Actually build the product
├── Need creativity, understanding, skills
├── Follow manager's resource decisions
└── Report results back
```

### The Key Insight

```
PID KNOWS:  "Error is 0.4"
PID ASKS:   "Is 0.4 > threshold?"
PID DOES:   Mathematical comparison → action

LLM KNOWS:  "Error is 0.4 because test_validation failed
            due to missing null check in line 42"
LLM ASKS:   "What's the best way to fix this?"
LLM DOES:   Writes the actual fix

INSIGHT:    Controller doesn't NEED to know WHY.
            It just needs to know WHAT to do about it.
            Agents already explain failures in their output.
```

### When Would LLM Controller Make Sense?

LLM controller might be useful if:
- Need to explain decisions to users ("Why did you swap the agent?")
- Very complex task routing requiring domain knowledge
- Low throughput (<10 decisions/hour) where latency doesn't matter
- Research/exploration phase where maximum intelligence is desired

**For this system:** PID is sufficient because agents handle complexity.

### Summary: Right Tool for Each Job

| Component | Tool | Why |
|-----------|------|-----|
| **TCP Controller** | PID (Math) | Fast, free, deterministic — just needs to react to numbers |
| **Test Generator** | LLM | Needs to understand task and create tests |
| **Code Generator** | LLM | Needs to write creative, working code |
| **Reviewer** | LLM | Needs to understand code and find issues |
| **Feedback Agent** | LLM | Needs to analyze results and summarize |

**This is good engineering: use the simplest tool that solves each problem.**
