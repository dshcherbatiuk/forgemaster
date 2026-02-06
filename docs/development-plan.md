# ForgeMaster Development Plan

**Project:** ForgeMaster — Meta-Agent System for AgentForge Hackathon 2026

**Team:** CSM-101

**Last Updated:** 2026-02-06

---

## Overview

This plan outlines the development phases for building ForgeMaster, a meta-agent system that discovers, creates, orchestrates, and manages task-specific agent subsystems in Kubernetes.

```
┌─────────────────────────────────────────────────────────────────┐
│                    DEVELOPMENT PHASES                            │
│                                                                  │
│   Phase 1        Phase 2        Phase 3        Phase 4          │
│   ─────────      ─────────      ─────────      ─────────        │
│   Foundation     Core Engine    Agents         Integration      │
│   & UI Portal   & Protocols                   & Demo           │
│                                                                  │
│   [██████████]   [░░░░░░░░░░]   [░░░░░░░░░░]   [░░░░░░░░░░]     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation & UI Portal

**Goal:** Set up infrastructure, build web portal, and deploy AgentTask CRD.

### 1.1 Development Environment

- [x] Initialize Rust workspace with crate structure
- [x] Set up Cargo workspace with shared dependencies
- [x] Configure CI/CD pipeline (GitHub Actions)
- [x] Set up linting (clippy) and formatting (rustfmt)

### 1.2 Local Kubernetes Cluster

- [x] Create Ansible playbook for OrbStack cluster setup
- [x] Configure kubectl and helm
- [x] Verify cluster is operational
- [x] Install Gateway API CRDs
- [x] Install Envoy Gateway controller
- [x] Create GatewayClass

### 1.3 UI Portal

#### Frontend Application (React + CopilotKit A2UI)
- [x] Set up React application with Vite
- [x] Style with Tailwind CSS
- [x] Integrate CopilotKit A2UI renderer (`@copilotkit/a2ui-renderer`)
- [x] Create dynamic JSON schema system with auto-discovery
- [x] Create component composition system (includes-based merging)
- [x] Create unified dashboard schema with reusable components:
  - [x] `dashboard.json` — Full layout with includes
  - [x] `components/header.json` — App header with title
  - [x] `components/navigation.json` — Navigation buttons
  - [x] `components/taskForm.json` — Task input with description field
  - [x] `components/progressStepper.json` — Shows phases: Analyze → Gen Tests → Gen Code → Run Tests → Complete
  - [x] `components/agentCard.json` — Agent status display (name, role, metrics)
  - [x] `components/testResults.json` — Test pass/fail/skip summary
  - [x] `components/tcpGauge.json` — Error signal visualization (0.0 - 1.0)
  - [ ] `components/logViewer.json` — Streaming log output
  - [ ] `components/artifactViewer.json` — Generated code/test display
  - [ ] `components/iterationTimeline.json` — Shows iteration history with error trend

### 1.4 Custom Resource Definitions (CRDs)

- [x] Define AgentTask CRD schema (Helm chart)
- [x] Define Agent CRD schema
- [ ] Define MCPServer CRD schema
- [x] Apply AgentTask CRD to cluster

### 1.5 Helm Charts & Deployment

- [x] Create Helm chart for UI (ui/helm/)
- [x] Define values.yaml with configurable options
- [x] Create templates (deployment, service, gateway, httproute)
- [x] Create Dockerfile for UI
- [x] Create Ansible roles (gateway, ui, fm-controller-agenttask, fm-controller-agent)
- [x] Create site.yml playbook
- [x] Deploy UI to cluster via Gateway API
- [x] Create Helm chart for AgentTask CRD (crates/fm-controller-agenttask/helm/)

**Deliverables:**
- Working local K8s cluster
- Web portal with A2UI rendering and WebSocket integration
- CRDs installed and validated
- Helm chart structure ready
- Rust workspace compiles

---

## Phase 2: Agent Runtime & Core Agents

**Goal:** Build the agent execution layer — runtime, first agents, and the controllers that manage them.

> **Why agents before TCP Controller?** The TCP Controller computes `error = failed_tests / total_tests`,
> but nothing produces test results until agents are running. Build the agents first, then wire the feedback loop.

### 2.1 AgentTask Controller (fm-controller-agenttask)

- [x] Define AgentTask CRD schema
- [x] Create Helm chart for CRD
- [x] Deploy CRD to cluster
- [x] Implement AgentTask CRD structs in Rust
- [x] Implement ReconcileStrategy trait pattern
- [x] Implement phase-based reconciliation (Pending, Clarifying, Running, Succeeded, Failed)
- [x] Implement DashMap dispatcher for strategy routing
- [x] Containerize and deploy to K8s
- [x] Implement WebSocket server with `/ws` endpoint (see [WS Protocol](arch/15-ws-protocol.md))
- [x] Implement connection registry (DashMap-based client tracking)
- [x] Implement WsEvent/WsCommand message types (snake_case JSON)
- [x] Implement SchemaCache for late-joiner data push
- [x] Implement UI WebSocket hook with auto-reconnect
- [x] Implement ConnectionStatus component
- [x] Add Helm templates (Service, HTTPRoute, ConfigMap)
- [x] Implement UI → Server action commands (typed WsCommand variants + ActionDispatcher)
- [x] Implement WsAction trait with DashMap-based action routing
- [x] Implement SubmitTaskAction (creates AgentTask CRD from UI)
- [x] Fix update_phase to patch only phase field (preserve existing status)
- [x] Implement broadcast channel (tokio::sync::broadcast) for controller → WS state propagation
- [x] Implement TaskStateBroadcaster (listens on channel, builds A2UI schema, pushes to clients)
- [x] Implement server-generated A2UI schemas (build_task_status_schema in Rust)
- [x] Add WsEvent::Schema variant for pushing components + data to UI
- [x] Implement ID-based schema merging in UI (server components override static by ID)
- [x] Read real CRD status values in update_phase (spec description + status fields from AgentTask)
- [x] Task Status card with full details (Name, Description, Phase, Iteration, Error Signal, Tests)
- [x] Late joiner support (SchemaCache sends cached schema + data on connect)
- [ ] Implement clarification relay (agent → UI → agent)
- [ ] Implement A2UI schema diff (push only changes)
- [x] Add namespace lifecycle management (create namespace on Pending→Running, delete on task deletion with finalizer)

### 2.2 Agent Runtime (fm-agent-runtime)

- [ ] Create base agent runtime crate
- [ ] Implement Claude API integration (streaming)
- [ ] Implement system prompt loading
- [ ] Add conversation management
- [ ] Add MCP client integration
- [ ] Create agent container image

### 2.3 Agent Controller (fm-controller-agent)

- [x] Define Agent CRD schema
- [x] Create Helm chart
- [ ] Implement Agent CRD structs in Rust
- [ ] Watch AgentTask CRs — create Orchestrator Agent CR when task enters Running
- [ ] Implement reconciliation loop (create/manage agent pods)
- [ ] Containerize and deploy

### 2.4 MCPServer Controller (fm-controller-mcpserver)

- [ ] Define MCPServer CRD schema
- [ ] Create Helm chart
- [ ] Implement MCPServer CRD structs in Rust
- [ ] Implement reconciliation loop
- [ ] Containerize and deploy

### 2.5 Core Agents

#### Orchestrator Agent (per-task Agent CR, created by Agent Controller)
- [ ] Implement task decomposition (analyze description, decide agents/MCPs)
- [ ] Create executor Agent CRs and MCPServer CRs via K8s API
- [ ] Implement agent coordination via A2A
- [ ] Test orchestration flow

#### Test Generator Agent
- [ ] Implement task analysis
- [ ] Add Gherkin feature generation
- [ ] Implement acceptance criteria extraction
- [ ] Test with sample tasks

#### Code Generator Agent
- [ ] Implement code generation from tests
- [ ] Add multi-language support (Rust, Python, TypeScript)
- [ ] Implement iterative refinement
- [ ] Test with Gherkin inputs

#### Test Runner Agent
- [ ] Implement Gherkin test execution (behave)
- [ ] Add result parsing
- [ ] Implement error extraction
- [ ] Test with generated code

#### Feedback Agent
- [ ] Implement metrics collection
- [ ] Add error signal calculation
- [ ] Implement pattern analysis
- [ ] Add recommendations generation
- [ ] Test feedback loop

**Deliverables:**
- Agent Runtime crate with Claude API integration
- Agent Controller and MCPServer Controller managing pods
- Core agents deployed and functional
- End-to-end flow: Submit → Generate Tests → Generate Code → Run Tests

---

## Phase 3: Feedback Loop & Protocols

**Goal:** Wire the TCP feedback loop and implement protocol stack for agent communication.

### 3.1 TCP Controller (fm-tcp-controller)

- [ ] Implement core PID-like control logic
- [ ] Create error signal calculation
- [ ] Implement control actions (swap, add, retry, complete)
- [ ] Add context history tracking
- [ ] Create internal API for orchestrator
- [ ] Add Redis integration for state persistence
- [ ] Write unit tests for control logic
- [ ] Containerize and deploy to K8s

### 3.2 Agent Registry

- [ ] Implement agent registration API
- [ ] Add heartbeat and health checking
- [ ] Implement skill-based search
- [ ] Add Redis storage backend
- [ ] Create deregistration and cleanup
- [ ] Write unit tests
- [ ] Containerize and deploy to K8s

### 3.3 MCP Protocol Implementation

- [ ] Create mcp-core crate (types, traits)
- [ ] Implement MCP client for agents
- [ ] Deploy official MCP servers:
  - [ ] filesystem-mcp
  - [ ] github-mcp
- [ ] Test tool discovery and execution

### 3.4 A2A Protocol Implementation

- [ ] Create a2a-core crate (types, traits)
- [ ] Implement A2A server (Axum-based)
- [ ] Implement A2A client
- [ ] Add Agent Card generation
- [ ] Implement SSE streaming for task updates
- [ ] Test agent-to-agent communication

**Deliverables:**
- TCP Controller running with PID feedback loop
- Agent Registry service running
- MCP servers deployed and functional
- A2A protocol working
- Full feedback loop: Agents → Test Results → TCP Controller → Adjustment → Agents

---

## Phase 4: Integration & Demo

**Goal:** Full system integration, testing, and hackathon demo preparation.

### 4.1 Context Management

- [ ] Implement context store (Redis)
- [ ] Add artifact storage (GitHub)
- [ ] Implement checkpoint/restore
- [ ] Test context persistence across iterations

### 4.2 End-to-End Testing

- [ ] Create integration test suite
- [ ] Test complete task lifecycle:
  - [ ] Task submission → Agent creation → Execution → Completion
- [ ] Test failure scenarios and recovery
- [ ] Performance testing

### 4.3 Demo Scenario

- [ ] Finalize demo task: "E-commerce backend API"
- [ ] Create demo script with talking points
- [ ] Record backup video
- [ ] Prepare slides for architecture explanation
- [ ] Test demo flow multiple times

### 4.4 Documentation

- [ ] Update README with quick start
- [ ] Create user guide
- [ ] Document API endpoints
- [ ] Add troubleshooting guide

**Deliverables:**
- Fully integrated system
- Demo-ready scenario
- Documentation complete
- Backup video recorded

---

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         FORGEMASTER ARCHITECTURE                         │
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                     UI (A2UIRenderer only)                       │   │
│   │              Renders A2UI schemas from agents                    │   │
│   └───────────────────────────┬─────────────────────────────────────┘   │
│                               │ WebSocket (A2UI schemas)                 │
│   ┌───────────────────────────▼─────────────────────────────────────┐   │
│   │                    CONTROL PLANE (forgemaster-system)            │   │
│   │  ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐       │   │
│   │  │AgentTask         │  │TCP Controller│  │Agent Registry│       │   │
│   │  │Controller        │  │  (PID Math)  │  │   (Redis)    │       │   │
│   │  │(WebSocket + K8s) │  └──────────────┘  └──────────────┘       │   │
│   │  │+ Schema Cache    │                                            │   │
│   │  └──────────────────┘                                            │   │
│   └─────────────────────────────┬───────────────────────────────────┘   │
│                                 │ A2A + A2UI schemas                     │
│   ┌─────────────────────────────▼───────────────────────────────────┐   │
│   │                    TASK NAMESPACE (task-xxxxx)                   │   │
│   │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │   │
│   │  │Orchestrator│ │Test Gen    │ │Code Gen    │ │Test Runner │   │   │
│   │  │   Agent    │ │  Agent     │ │  Agent     │ │   Agent    │   │   │
│   │  │ + A2UI Gen │ │ + A2UI Gen │ │ + A2UI Gen │ │            │   │   │
│   │  └─────┬──────┘ └────────────┘ └────────────┘ └────────────┘   │   │
│   │        │ MCP                                                     │   │
│   │  ┌─────▼──────────────────────────────────────────────────────┐ │   │
│   │  │              MCP SERVERS                                    │ │   │
│   │  │  [github-mcp] [postgres-mcp] [stripe-mcp] [filesystem-mcp] │ │   │
│   │  └────────────────────────────────────────────────────────────┘ │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### A2UI Data Flow

```
┌─────────┐    ┌────────────────────┐    ┌─────────────────┐
│  Agent  │───▶│ AgentTask Controller│───▶│ UI (A2UIRenderer)│
│         │    │                    │    │                 │
│ Generate│    │ • Cache schema     │    │ • Render schema │
│ A2UI    │    │ • Compute diff     │    │ • User input    │
│ schema  │    │ • Push via WS      │    │                 │
└─────────┘    └────────────────────┘    └─────────────────┘
     │                  ▲                         │
     │                  │                         │
     └──────────────────┴─────────────────────────┘
              Clarification answers via WebSocket
```

---

## Crate Structure

Each crate follows the pattern: crate + helm chart + ansible role per responsibility.

```
forgemaster/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── fm-core/                  # Shared types, traits, config
│   │
│   ├── fm-controller-agenttask/  # AgentTask CRD + Controller
│   │   ├── helm/                 # Helm chart for this service
│   │   └── src/
│   │
│   ├── fm-controller-agent/      # Agent CRD + Controller (future)
│   │   └── helm/
│   │
│   ├── fm-controller-mcpserver/  # MCPServer CRD + Controller (future)
│   │   └── helm/
│   │
│   ├── fm-tcp-controller/        # TCP Controller service (future)
│   │   └── helm/
│   │
│   ├── fm-agent-registry/        # Agent Registry service (future)
│   │   └── helm/
│   │
│   ├── fm-agent-runtime/         # Base agent execution runtime (future)
│   │
│   ├── fm-a2a/                   # A2A protocol (future)
│   │
│   └── fm-mcp/                   # MCP protocol (future)
│
├── ui/                           # React + CopilotKit A2UI portal
│   ├── helm/                     # UI Helm chart
│   └── src/
│
├── ansible/                      # Infrastructure automation
│   ├── site.yml
│   └── roles/
│       ├── gateway/
│       ├── ui/
│       └── fm-controller-agenttask/
│
└── docs/                         # Documentation
```

---

## Technology Stack

| Layer | Technology |
|-------|------------|
| **Language** | Rust |
| **Web Framework** | Axum |
| **Async Runtime** | Tokio |
| **K8s Operator** | kube-rs |
| **State Store** | Redis |
| **Artifacts** | GitHub (via MCP) |
| **LLM** | Claude API |
| **Frontend** | React + CopilotKit A2UI |
| **UI Protocol** | A2UI |
| **Agent Protocol** | A2A |
| **Tool Protocol** | MCP |
| **Container Runtime** | Docker |
| **Orchestration** | Kubernetes (OrbStack) |
| **Package Manager** | Helm |
| **Infra Automation** | Ansible |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| K8s complexity | Start with Kind locally; use Helm for repeatability |
| Claude API rate limits | Implement retry with backoff; cache responses |
| A2A/A2UI spec changes | Pin versions; abstract behind interfaces |
| Time constraints | Prioritize MVP; cut features if needed |
| Demo failures | Prepare fallback scenarios; record backup video |

---

## MVP Scope (Hackathon)

**Must Have:**
- [ ] Web portal with live task status (Phase 1)
- [ ] TCP Controller with PID logic
- [ ] Agent Registry with basic registration
- [ ] Orchestrator + Test Generator + Code Generator agents
- [ ] Single working demo: E-commerce API generation

**Nice to Have:**
- [ ] Outcome validation
- [ ] Full A2UI component catalog
- [ ] Multiple MCP servers

**Out of Scope (Post-Hackathon):**
- [ ] Multi-cluster support
- [ ] Fine-tuning / learning
- [ ] Production hardening
- [ ] Multi-tenant isolation

---

## Next Steps

1. ~~**Initialize Rust workspace** — Set up crate structure~~ ✅
2. ~~**Create OrbStack cluster** — Ansible playbook for local K8s~~ ✅
3. ~~**Build UI Portal** — React + CopilotKit A2UI~~ ✅
4. ~~**AgentTask Controller** — CRD, WebSocket, live status~~ ✅
5. **Agent Runtime** — Base crate with Claude API integration
6. **Agent Controller** — Watch Agent CRs, manage pods
7. **Orchestrator Agent** — First real agent, task decomposition
8. **TCP Controller** — PID feedback loop (needs agent test results)
