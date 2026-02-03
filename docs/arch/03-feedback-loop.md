## Feedback Loop Flow

```mermaid
flowchart TD
    A[1. Task arrives] --> B[2. Test Generator creates E2E tests]
    B -->|"setpoint defined"| C[3. Orchestrator selects/creates agents]
    C --> D[4. Executor agents perform task]
    D --> E[5. Test Runner validates output]
    E --> F[6. Feedback Collector calculates error]
    F --> G{7. Error > threshold?}
    G -->|Yes| H[Controller adjusts strategy]
    H --> C
    G -->|No| I[8. Task complete, return output]
```

---

## TCP Control Logic

```mermaid
stateDiagram-v2
    [*] --> Analyzing: Task received
    
    Analyzing --> HighError: error > 0.5
    Analyzing --> MediumError: 0.2 < error ≤ 0.5
    Analyzing --> LowError: error ≤ 0.2
    
    HighError --> MajorChange: T action
    MajorChange --> Analyzing: Swap agent / change approach
    
    MediumError --> ModerateChange: T action
    ModerateChange --> Analyzing: Add reviewer / adjust params
    
    LowError --> MinorChange: T action
    MinorChange --> Complete: Fine-tune / retry
    
    Complete --> [*]: Task done
```

### T (Task) — Immediate Response

```python
t_action = Kt * current_error
```

| Error Level | Action |
|-------------|--------|
| High (> 0.5) | Major change: swap agent, change approach |
| Medium (0.2-0.5) | Moderate: add reviewer agent, adjust params |
| Low (< 0.2) | Minor: fine-tune, retry failed tests |

### C (Context) — Accumulated Learning

```python
context += current_error * dt
c_action = Kc * context
```

Tracks patterns over time:
- Same task type keeps failing → flag for model change
- Certain agents consistently underperform → deprioritize
- Successful patterns → remember and reuse

### P (Prediction) — Predictive Adjustment

```python
prediction = (current_error - previous_error) / dt
p_action = Kp * prediction
```

| Trend | Meaning | Action |
|-------|---------|--------|
| Error increasing | Getting worse | Preemptive intervention |
| Error stable | No progress | Try different approach |
| Error decreasing | Improving | Continue current strategy |

---
