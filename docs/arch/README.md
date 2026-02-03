# Meta-Agent Architecture Documentation

> Autonomous meta-agent: analyzes tasks, creates/selects agents & MCP servers, orchestrates execution

This directory contains the architecture documentation for the Meta-Agent System, split by responsibility.

## Document Structure

```mermaid
flowchart TB
    subgraph Core["Core Architecture"]
        OV[01-overview.md]
        COMP[02-components.md]
        FB[03-feedback-loop.md]
    end

    subgraph ADR["ADRs (../adr/)"]
        DD[0001-tcp-controller-vs-llm-agents]
    end

    subgraph Deployment["Deployment"]
        K8S[05-k8s-deployment.md]
        HELM[11-helm-charts.md]
        INFRA[10-infrastructure.md]
    end

    subgraph Protocols["Protocols"]
        A2A[08-a2a-protocol.md]
        A2UI[09-a2ui-protocol.md]
        REG[07-agent-registry.md]
    end

    subgraph Services["Services"]
        RUST[13-backend-services.md]
        STORE[12-artifact-storage.md]
        TECH[14-tech-stack.md]
    end

    subgraph Future["Vision"]
        AGI[15-agi-roadmap.md]
        EDGE[06-edge-cases.md]
        SUM[16-summary.md]
    end
```

## Documents

### Core Architecture
| File | Description |
|------|-------------|
| [01-overview.md](01-overview.md) | System overview and core concepts (TCP Controller, Gherkin setpoints, error signal) |
| [02-components.md](02-components.md) | Architecture diagram and component descriptions |
| [03-feedback-loop.md](03-feedback-loop.md) | Feedback loop flow and TCP control logic |

### Architecture Decision Records
| File | Description |
|------|-------------|
| [0001-tcp-controller-vs-llm-agents.md](../adr/0001-tcp-controller-vs-llm-agents.md) | Why TCP (PID) Controller + LLM Agents (key architectural decision) |

### Deployment & Infrastructure
| File | Description |
|------|-------------|
| [05-k8s-deployment.md](05-k8s-deployment.md) | Kubernetes deployment model and CRDs |
| [10-infrastructure.md](10-infrastructure.md) | Ansible playbooks and local K8s cluster setup |
| [11-helm-charts.md](11-helm-charts.md) | Helm chart structure and configuration |

### Protocols & Communication
| File | Description |
|------|-------------|
| [07-agent-registry.md](07-agent-registry.md) | Agent registry for discovery and health management |
| [08-a2a-protocol.md](08-a2a-protocol.md) | Agent-to-Agent (A2A) communication protocol |
| [09-a2ui-protocol.md](09-a2ui-protocol.md) | Agent-to-User Interface (A2UI) protocol |

### Backend Services
| File | Description |
|------|-------------|
| [12-artifact-storage.md](12-artifact-storage.md) | GitHub-based artifact storage |
| [13-backend-services.md](13-backend-services.md) | Rust backend services implementation |
| [14-tech-stack.md](14-tech-stack.md) | Technology stack overview |

### Vision & Future
| File | Description |
|------|-------------|
| [06-edge-cases.md](06-edge-cases.md) | Edge cases, mitigations, and future considerations |
| [15-agi-roadmap.md](15-agi-roadmap.md) | Relation to AGI and roadmap for extension |
| [16-summary.md](16-summary.md) | System summary and architecture overview |

## Quick Start

1. Start with **[01-overview.md](01-overview.md)** for core concepts
2. Review **[ADR-0001](../adr/0001-tcp-controller-vs-llm-agents.md)** to understand the key TCP Controller vs LLM split
3. Explore **[05-k8s-deployment.md](05-k8s-deployment.md)** for deployment details
4. Check **[13-backend-services.md](13-backend-services.md)** for Rust implementation patterns

## Key Concepts

- **TCP Controller**: PID-inspired orchestration (Task-Context-Prediction)
- **Error Signal**: `error = failed_tests / total_tests`
- **Gherkin Setpoint**: E2E tests define dynamic success criteria
- **A2A/A2UI/MCP**: Protocol stack for agent communication
