## Edge Cases & Mitigations

| Problem | Mitigation |
|---------|------------|
| Test Agent writes bad tests | Validation layer, multiple test agents with voting |
| Tests too strict/loose | Confidence scoring on tests |
| Infinite loop | Max iterations + fallback to human review |
| Non-testable tasks | Hybrid: E2E for code, LLM-as-judge for creative |
| Agent keeps failing | Circuit breaker pattern, blacklist agent |
| Resource exhaustion | K8s resource limits, pod priority |

---

## Future Considerations

- [ ] Multi-task parallelism
- [ ] Agent marketplace / registry
- [ ] Learning from successful runs (fine-tuning)
- [ ] Cost optimization (cheap agents first, expensive as fallback)
- [ ] Human-in-the-loop for low-confidence decisions

---
