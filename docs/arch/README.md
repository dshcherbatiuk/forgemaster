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

    subgraph TaskExec["Task Execution"]
        TL[04-task-lifecycle.md]
        AC[04a-agent-creation.md]
        OS[04b-orchestrator-spec.md]
        SR[04c-domain-reuse.md]
        CM[04d-context-management.md]
    end

    subgraph ADR["ADRs (../adr/)"]
        DD[0001-tcp-controller-vs-llm-agents]
    end

    subgraph Deployment["Deployment"]
        K8S[05-k8s-deployment.md]
        HELM[10-helm-charts.md]
        INFRA[09-infrastructure.md]
    end

    subgraph Protocols["Protocols"]
        REG[06-agent-registry.md]
        A2A[07-a2a-protocol.md]
        A2UI[08-a2ui-protocol.md]
    end

    subgraph Services["Services"]
        STORE[11-artifact-storage.md]
        RUST[12-backend-services.md]
        TECH[13-tech-stack.md]
    end

    subgraph Future["Vision"]
        AGI[14-agi-roadmap.md]
    end

    Core --> TaskExec
    TaskExec --> Deployment
```

## Documents

### Core Architecture
| File | Description |
|------|-------------|
| [01-overview.md](01-overview.md) | System overview and core concepts (TCP Controller, Gherkin setpoints, error signal) |
| [02-components.md](02-components.md) | Architecture diagram and component descriptions |
| [03-feedback-loop.md](03-feedback-loop.md) | Feedback loop flow and TCP control logic |

### Task Execution
| File | Description |
|------|-------------|
| [04-task-lifecycle.md](04-task-lifecycle.md) | Task submission, state machine, namespace lifecycle |
| [04a-agent-creation.md](04a-agent-creation.md) | Agent instantiation algorithm, templates, custom agents |
| [04b-orchestrator-spec.md](04b-orchestrator-spec.md) | Orchestrator decision logic, signal interpretation |
| [04c-domain-reuse.md](04c-domain-reuse.md) | Domain templates, matching, composition |
| [04d-context-management.md](04d-context-management.md) | Context schema, flow between agents, API |

### Architecture Decision Records
| File | Description |
|------|-------------|
| [0001-tcp-controller-vs-llm-agents.md](../adr/0001-tcp-controller-vs-llm-agents.md) | Why TCP (PID) Controller + LLM Agents (key architectural decision) |

### Deployment & Infrastructure
| File | Description |
|------|-------------|
| [05-k8s-deployment.md](05-k8s-deployment.md) | Kubernetes deployment model and CRDs |
| [09-infrastructure.md](09-infrastructure.md) | Ansible playbooks and local K8s cluster setup |
| [10-helm-charts.md](10-helm-charts.md) | Helm chart structure and configuration |

### Protocols & Communication
| File | Description |
|------|-------------|
| [06-agent-registry.md](06-agent-registry.md) | Agent registry for discovery and health management |
| [07-a2a-protocol.md](07-a2a-protocol.md) | Agent-to-Agent (A2A) communication protocol |
| [08-a2ui-protocol.md](08-a2ui-protocol.md) | Agent-to-User Interface (A2UI) protocol |

### Backend Services
| File | Description |
|------|-------------|
| [11-artifact-storage.md](11-artifact-storage.md) | GitHub-based artifact storage |
| [12-backend-services.md](12-backend-services.md) | Rust backend services implementation |
| [13-tech-stack.md](13-tech-stack.md) | Technology stack overview |

### Vision & Future
| File | Description |
|------|-------------|
| [14-agi-roadmap.md](14-agi-roadmap.md) | Relation to AGI and roadmap for extension |

## Quick Start

1. Start with **[Project Goal](../project-goal.md)** for hackathon context and vision
2. Read **[01-overview.md](01-overview.md)** for core concepts
3. Review **[ADR-0001](../adr/0001-tcp-controller-vs-llm-agents.md)** to understand the key TCP Controller vs LLM split
4. Explore **[04-task-lifecycle.md](04-task-lifecycle.md)** for how tasks flow through the system
5. Check **[04c-domain-reuse.md](04c-domain-reuse.md)** for domain reuse patterns
6. See **[05-k8s-deployment.md](05-k8s-deployment.md)** for deployment details

## Key Concepts

- **TCP Controller**: Task-Context-Prediction orchestration
- **Error Signal**: `error = (failed_tests + failed_outcomes) / total_checks`
- **Gherkin Setpoint**: E2E tests define dynamic success criteria
- **A2A/A2UI/MCP**: Protocol stack for agent communication
