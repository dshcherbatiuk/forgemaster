## Tech Stack (Proposed)

| Component | Technology |
|-----------|------------|
| **Backend Language** | **Rust** |
| Infrastructure Automation | Ansible |
| Local K8s Cluster | Kind (Kubernetes in Docker) |
| Orchestration | Kubernetes + Custom CRDs |
| Package Management | Helm Charts |
| K8s Operator | kube-rs (Rust) |
| Web Framework | Axum (Rust) |
| Async Runtime | Tokio |
| Agent Registry | Rust (Axum + Redis) |
| TCP Controller | Rust (Axum + Tokio) |
| Agent Runtime | Rust (async-openai) |
| A2A Implementation | a2a-rs (Rust) |
| A2UI Implementation | a2ui-rs (Rust) |
| MCP Client | Rust |
| Agent ↔ User | A2UI Protocol |
| Agent ↔ Agent | A2A Protocol |
| Agent ↔ Tools | MCP Protocol |
| A2UI Client | React / Flutter |
| Test Framework | Behave (Gherkin) |
| State Store | Redis |
| Artifact Storage | GitHub (via MCP) |
| Observability | OpenTelemetry (Rust SDK) |
| LLM | Claude API |

---
