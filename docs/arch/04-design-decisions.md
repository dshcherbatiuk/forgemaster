## Design Decision: Why TCP (PID) Controller + LLM Agents

A key architectural decision in this system is the separation of concerns between the **controller** and **agents**:

- **TCP Controller** — Uses PID-like mathematical control (no LLM)
- **Agents** — Use LLM for creative work (Claude API)

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     TCP CONTROLLER (PID)                        │
│                         Rust + Math                             │
│                                                                 │
│   Role: ORCHESTRATION — "What to do"                           │
│   • Measure error signal                                        │
│   • Compute control action (swap, retry, add agent)            │
│   • Fast, deterministic, free                                   │
└─────────────────────────────┬───────────────────────────────────┘
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

### Why NOT Use LLM for Controller?

| Aspect | PID Controller | LLM Controller |
|--------|----------------|----------------|
| **Speed** | 0.001ms | 2000ms (2M× slower) |
| **Cost** | $0 | ~$0.01/decision |
| **Determinism** | 100% same input → same output | Non-deterministic |
| **Reliability** | Always works | API can fail/timeout |
| **Scalability** | 1M decisions/sec | ~1 decision/sec |
| **Debuggability** | Easy to trace math | Hard to explain |

### What Controller Actually Needs

The controller doesn't need to **understand** — it just needs to **react**:

```
┌─────────────────────────────────────────────────────────────┐
│                    CONTROLLER INPUT                          │
│                                                              │
│   • error = 0.4 (number)                                    │
│   • previous_error = 0.5 (number)                           │
│   • iteration = 3 (number)                                  │
│                                                              │
│   Question: "Should I swap agent, add agent, or continue?"  │
│                                                              │
│   This is a MATHEMATICAL decision, not a CREATIVE one.      │
└─────────────────────────────────────────────────────────────┘
```

PID is perfect for this:
```rust
// Controller decision - pure math, no understanding needed
let signal = Kt * error + Kc * integral + Kp * derivative;

match signal {
    s if s > 0.5 => SwapAgent,    // High error → drastic change
    s if s > 0.2 => AddAgent,     // Medium error → add help
    _ => Continue,                 // Low error → keep going
}
```

### What Agents Actually Need

Agents need to **understand** and **create** — this requires LLM:

```
┌─────────────────────────────────────────────────────────────┐
│                      AGENT INPUT                             │
│                                                              │
│   • Task: "Build REST API for user management"              │
│   • Context: Previous attempts, test failures               │
│   • Requirements: Must pass Gherkin tests                   │
│                                                              │
│   Question: "How do I implement this?"                      │
│                                                              │
│   This is a CREATIVE decision requiring UNDERSTANDING.      │
└─────────────────────────────────────────────────────────────┘
```

LLM is essential for this:
```rust
// Agent work - requires understanding and creativity
let prompt = format!(
    "Generate Rust code for REST API that passes these tests:\n{}",
    gherkin_tests
);
let code = llm.complete(&prompt).await?; // Creative work
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

### Comparison by Scenario

**Scenario 1: Error Decreasing (0.6 → 0.4 → 0.2)**

| Controller | Decision | Time | Cost |
|------------|----------|------|------|
| PID | Continue (derivative negative) | 0.001ms | $0 |
| LLM | "Error decreasing, continue" | 2000ms | $0.01 |

→ **Same result, PID is 2M× faster and free**

**Scenario 2: High Error (0.8)**

| Controller | Decision | Time | Cost |
|------------|----------|------|------|
| PID | Swap agent (error > 0.5) | 0.001ms | $0 |
| LLM | "High error, analyzing why... swap agent" | 2000ms | $0.01 |

→ **Same result, PID is faster**

**Scenario 3: Stuck Error (0.4 → 0.4 → 0.4)**

| Controller | Decision | Quality |
|------------|----------|---------|
| PID | Swap agent (threshold) | Generic but works |
| LLM | "Same tests failing, add validation-specialist" | More targeted |

→ **LLM is smarter, but PID still solves the problem**

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

**Conclusion:** Save LLM budget for agents who need it!

### The Key Insight

```
┌────────────────────────────────────────────────────────────────┐
│                                                                 │
│   PID KNOWS:  "Error is 0.4"                                   │
│   PID ASKS:   "Is 0.4 > threshold?"                            │
│   PID DOES:   Mathematical comparison → action                  │
│                                                                 │
│   ─────────────────────────────────────────────────────────    │
│                                                                 │
│   LLM KNOWS:  "Error is 0.4 because test_validation failed     │
│               due to missing null check in line 42"            │
│   LLM ASKS:   "What's the best way to fix this?"               │
│   LLM DOES:   Writes the actual fix                            │
│                                                                 │
│   ─────────────────────────────────────────────────────────    │
│                                                                 │
│   INSIGHT:    Controller doesn't NEED to know WHY.             │
│               It just needs to know WHAT to do about it.       │
│               Agents already explain failures in their output. │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### When Would LLM Controller Make Sense?

LLM controller might be useful if:
- Need to explain decisions to users ("Why did you swap the agent?")
- Very complex task routing requiring domain knowledge
- Low throughput (<10 decisions/hour) where latency doesn't matter
- Research/exploration phase where you want maximum intelligence

**But for this system:** PID is sufficient because agents handle complexity.

### Final Architecture Decision

```yaml
# TCP Controller: PID (Rust)
tcp_controller:
  type: pid  # NOT llm
  implementation: rust
  latency: "<1ms"
  cost: "$0"
  coefficients:
    task: 1.0
    context: 0.5
    prediction: 0.3

# Agents: LLM (Claude)
agents:
  type: llm  # Needs understanding
  implementation: claude-api
  latency: "1-5s"
  cost: "~$0.01/call"
  
  instances:
    - test_generator    # Creative: writes Gherkin
    - code_generator    # Creative: writes code
    - reviewer          # Creative: finds issues
    - feedback          # Creative: summarizes results
```

### Summary: Right Tool for Each Job

| Component | Tool | Why |
|-----------|------|-----|
| **TCP Controller** | PID (Math) | Fast, free, deterministic — just needs to react to numbers |
| **Test Generator** | LLM | Needs to understand task and create tests |
| **Code Generator** | LLM | Needs to write creative, working code |
| **Reviewer** | LLM | Needs to understand code and find issues |
| **Feedback Agent** | LLM | Needs to analyze results and summarize |

This separation gives us:
- ⚡ **Speed** — Controller decisions in microseconds
- 💰 **Cost efficiency** — LLM only where creativity is needed
- 🎯 **Reliability** — Deterministic orchestration
- 🧠 **Intelligence** — Smart agents where it matters

**This is good engineering: use the simplest tool that solves each problem.**

---
