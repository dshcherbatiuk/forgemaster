# ForgeMaster Development Plan

**Project:** ForgeMaster — Meta-Agent System for AgentForge Hackathon 2026

**Team:** CSM-101

**Last Updated:** 2026-02-05

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

**Goal:** Set up infrastructure and build web portal with dry-run mode to visualize the system early.

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

### 1.3 UI Portal (Dry-Run Mode)

Build the web portal early with mock data to visualize the complete flow before backend is ready.

#### Frontend Application (React + CopilotKit A2UI)
- [x] Set up React application with Vite
- [x] Style with Tailwind CSS
- [x] Integrate CopilotKit A2UI renderer (`@copilotkit/a2ui-renderer`)
- [x] Create dynamic JSON schema system with auto-discovery
- [x] Create ForgeMaster A2UI schemas (JSON):
  - [x] `taskForm.json` — Task input with description field
  - [x] `progressStepper.json` — Shows phases: Analyzing → Gen Tests → Gen Code → Running → Complete
  - [x] `agentCard.json` — Agent status display (name, role, metrics)
  - [x] `testResultsPanel.json` — Test pass/fail/skip summary
  - [x] `tcpGauge.json` — Error signal visualization (0.0 - 1.0)
  - [ ] `logViewer.json` — Streaming log output
  - [ ] `artifactViewer.json` — Generated code/test display
  - [ ] `iterationTimeline.json` — Shows iteration history with error trend

#### Dry-Run Mock Service
- [ ] Create mock REST API endpoints
- [ ] Implement mock SSE streaming with realistic timing
- [ ] Create sample mock data:
  - [ ] E-commerce task submission
  - [ ] Generated Gherkin tests
  - [ ] Generated Rust code
  - [ ] Test results (iterations showing improvement)
  - [ ] Agent status changes
  - [ ] TCP Controller decisions
- [ ] Add "dry-run" toggle in UI

#### Mock Data Scenarios
```
Scenario: E-commerce Backend API
├── Iteration 1: error=0.6 (3/5 tests pass)
│   └── TCP Decision: "Add reviewer agent"
├── Iteration 2: error=0.4 (4/5 tests pass)
│   └── TCP Decision: "Minor adjustment"
├── Iteration 3: error=0.2 (4/5 tests pass)
│   └── TCP Decision: "Continue"
└── Iteration 4: error=0.0 (5/5 tests pass)
    └── TCP Decision: "Complete - publish to registry"
```

### 1.4 Custom Resource Definitions (CRDs)

- [ ] Define AgentTask CRD schema
- [ ] Define Agent CRD schema
- [ ] Define MCPServer CRD schema
- [ ] Define Domain CRD schema
- [ ] Generate CRD manifests with kube-rs
- [ ] Apply CRDs to cluster

### 1.5 Helm Charts & Deployment

- [x] Create Helm chart for UI (ui/helm/)
- [x] Define values.yaml with configurable options
- [x] Create templates (deployment, service, gateway, httproute)
- [x] Create Dockerfile for UI
- [x] Create Ansible roles (gateway, ui)
- [x] Create site.yml playbook
- [x] Deploy UI to cluster via Gateway API
- [ ] Create Helm charts for other services

**Deliverables:**
- Working local K8s cluster
- **Web portal with dry-run mode showing complete UX flow**
- CRDs installed and validated
- Helm chart structure ready
- Rust workspace compiles

---

## Phase 2: Core Engine & Protocols

**Goal:** Implement TCP Controller, Agent Registry, and protocol stack. Connect to web portal.

### 2.1 REST API Service

- [ ] Create Axum-based REST API
- [ ] Implement task submission endpoint (`POST /api/v1/tasks`)
- [ ] Add SSE progress streaming (`GET /api/v1/tasks/{id}/progress`)
- [ ] Implement result retrieval (`GET /api/v1/tasks/{id}/result`)
- [ ] Add health check endpoint
- [ ] Connect web portal to real API (replace mock)
- [ ] Containerize and deploy to K8s

### 2.2 TCP Controller

- [ ] Implement core PID-like control logic
- [ ] Create error signal calculation
- [ ] Implement control actions (swap, add, retry, complete)
- [ ] Add context history tracking
- [ ] Create internal API for orchestrator
- [ ] Add Redis integration for state persistence
- [ ] Write unit tests for control logic
- [ ] Containerize and deploy to K8s

### 2.3 Agent Registry

- [ ] Implement agent registration API
- [ ] Add heartbeat and health checking
- [ ] Implement skill-based search
- [ ] Add Redis storage backend
- [ ] Create deregistration and cleanup
- [ ] Write unit tests
- [ ] Containerize and deploy to K8s

### 2.4 MCP Protocol Implementation

- [ ] Create mcp-core crate (types, traits)
- [ ] Implement MCP client for agents
- [ ] Deploy official MCP servers:
  - [ ] filesystem-mcp
  - [ ] github-mcp
- [ ] Test tool discovery and execution

### 2.5 A2A Protocol Implementation

- [ ] Create a2a-core crate (types, traits)
- [ ] Implement A2A server (Axum-based)
- [ ] Implement A2A client
- [ ] Add Agent Card generation
- [ ] Implement SSE streaming for task updates
- [ ] Test agent-to-agent communication

### 2.6 Kubernetes Operator

- [ ] Implement AgentTask controller
- [ ] Implement Agent controller
- [ ] Implement MCPServer controller
- [ ] Add namespace lifecycle management
- [ ] Implement reconciliation loops
- [ ] Add status updates for CRDs
- [ ] Containerize and deploy

**Deliverables:**
- TCP Controller service running
- Agent Registry service running
- REST API connected to web portal
- MCP servers deployed and functional
- A2A protocol working
- K8s Operator managing CRDs

---

## Phase 3: Agents

**Goal:** Implement core agents and connect to the system.

### 3.1 Agent Runtime

- [ ] Create base agent runtime crate
- [ ] Implement Claude API integration
- [ ] Add MCP client integration
- [ ] Add A2A server integration
- [ ] Implement system prompt loading
- [ ] Add conversation management
- [ ] Create agent container image

### 3.2 Core Agents

#### Orchestrator Agent
- [ ] Implement agent selection logic
- [ ] Add MCP server provisioning
- [ ] Implement K8s CRD management
- [ ] Add task decomposition
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
- All core agents deployed and functional
- Real task execution working
- End-to-end flow: Submit → Generate Tests → Generate Code → Run Tests → Complete

---

## Phase 4: Integration & Demo

**Goal:** Full system integration, testing, and hackathon demo preparation.

### 4.1 Domain Templates

- [ ] Create e-commerce domain template
- [ ] Create data-pipeline domain template
- [ ] Implement domain matching logic
- [ ] Test template reuse

### 4.2 Context Management

- [ ] Implement context store (Redis)
- [ ] Add artifact storage (GitHub)
- [ ] Implement checkpoint/restore
- [ ] Test context persistence across iterations

### 4.3 End-to-End Testing

- [ ] Create integration test suite
- [ ] Test complete task lifecycle:
  - [ ] Task submission → Agent creation → Execution → Completion
- [ ] Test failure scenarios and recovery
- [ ] Test domain reuse
- [ ] Performance testing

### 4.4 Demo Scenario

- [ ] Finalize demo task: "E-commerce backend API"
- [ ] Create demo script with talking points
- [ ] Record backup video
- [ ] Prepare slides for architecture explanation
- [ ] Test demo flow multiple times
- [ ] Prepare dry-run mode as fallback

### 4.5 Documentation

- [ ] Update README with quick start
- [ ] Create user guide
- [ ] Document API endpoints
- [ ] Add troubleshooting guide

**Deliverables:**
- Fully integrated system
- Demo-ready scenario
- **Dry-run mode as backup demo**
- Documentation complete
- Backup video recorded

---

## UI Portal Screens (Dry-Run Preview)

### Screen 1: Task Submission
```
┌─────────────────────────────────────────────────────────────────┐
│  FORGEMASTER                                    [Dry-Run: ON]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Create New Task                                                │
│  ─────────────────────────────────────────────                  │
│                                                                  │
│  Description:                                                    │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Create an e-commerce backend API with:                   │   │
│  │ - Product catalog with categories                        │   │
│  │ - Shopping cart management                               │   │
│  │ - Checkout with Stripe integration                       │   │
│  │ - Order history                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Language: [Rust ▼]    Framework: [Axum ▼]                      │
│                                                                  │
│                              [Submit Task]                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Screen 2: Progress View
```
┌─────────────────────────────────────────────────────────────────┐
│  FORGEMASTER                          Task: ecom-abc123         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Progress: ████████████░░░░░░░░ 60%        Iteration: 2/5       │
│                                                                  │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐      │
│  │ Analyze  │ Generate │ Generate │   Run    │ Complete │      │
│  │   Task   │  Tests   │   Code   │  Tests   │          │      │
│  │    ✓     │    ✓     │    ●     │    ○     │    ○     │      │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘      │
│                                                                  │
│  ┌─────────────────────────────────┬───────────────────────────┐│
│  │ TCP Controller                  │ Agents                    ││
│  │ ─────────────────────────       │ ───────────────────────   ││
│  │ Error Signal: ████░░░░ 0.40     │ [●] Orchestrator  Ready   ││
│  │                                 │ [●] TestGenerator Done    ││
│  │ Decision: "Continue iteration"  │ [◐] CodeGenerator Working ││
│  │                                 │ [○] TestRunner    Pending ││
│  │ History:                        │ [○] Feedback      Pending ││
│  │  Iter 1: 0.60 → Add reviewer    │                           ││
│  │  Iter 2: 0.40 → Continue        │                           ││
│  └─────────────────────────────────┴───────────────────────────┘│
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ Logs                                                   [▼]  ││
│  │ ─────────────────────────────────────────────────────────   ││
│  │ 10:23:45 [CodeGenerator] Generating product catalog API...  ││
│  │ 10:23:47 [CodeGenerator] Creating cart management endpoints ││
│  │ 10:23:49 [CodeGenerator] Implementing Stripe checkout...    ││
│  │ █                                                            ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Screen 3: Results View
```
┌─────────────────────────────────────────────────────────────────┐
│  FORGEMASTER                          Task: ecom-abc123  ✓      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ✓ Task Completed Successfully                                  │
│                                                                  │
│  ┌─────────────────────────────────┬───────────────────────────┐│
│  │ Test Results                    │ Artifacts                 ││
│  │ ─────────────────────────       │ ───────────────────────   ││
│  │ ✓ Add product to cart           │ 📁 src/                   ││
│  │ ✓ View shopping cart            │    ├── main.rs            ││
│  │ ✓ Process checkout              │    ├── cart.rs            ││
│  │ ✓ Handle invalid payment        │    ├── checkout.rs        ││
│  │ ✓ View order history            │    ├── products.rs        ││
│  │                                 │    └── stripe.rs          ││
│  │ 5/5 scenarios passed            │ 📁 tests/                 ││
│  │                                 │    └── api.feature        ││
│  └─────────────────────────────────┴───────────────────────────┘│
│                                                                  │
│  Iterations: 4    Total Time: 3m 24s    Tokens: 45,230          │
│                                                                  │
│  [View PR on GitHub]  [Download Artifacts]  [Run Again]         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         FORGEMASTER ARCHITECTURE                         │
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                        WEB PORTAL                                │   │
│   │                  (React + CopilotKit A2UI)                      │   │
│   └─────────────────────────────┬───────────────────────────────────┘   │
│                                 │ A2UI                                   │
│   ┌─────────────────────────────▼───────────────────────────────────┐   │
│   │                      REST API SERVICE                            │   │
│   │                         (Axum)                                   │   │
│   └─────────────────────────────┬───────────────────────────────────┘   │
│                                 │ K8s API                                │
│   ┌─────────────────────────────▼───────────────────────────────────┐   │
│   │                    CONTROL PLANE (forgemaster-system)            │   │
│   │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │   │
│   │  │TCP Controller│  │Agent Registry│  │  K8s Operator │           │   │
│   │  │  (PID Math)  │  │   (Redis)    │  │   (kube-rs)   │           │   │
│   │  └──────────────┘  └──────────────┘  └──────────────┘           │   │
│   └─────────────────────────────┬───────────────────────────────────┘   │
│                                 │ A2A                                    │
│   ┌─────────────────────────────▼───────────────────────────────────┐   │
│   │                    TASK NAMESPACE (task-xxxxx)                   │   │
│   │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │   │
│   │  │Orchestrator│ │Test Gen    │ │Code Gen    │ │Test Runner │   │   │
│   │  │   Agent    │ │  Agent     │ │  Agent     │ │   Agent    │   │   │
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

---

## Crate Structure

```
forgemaster/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── fm-core/                  # Shared types, traits, config
│   ├── tcp-controller/           # TCP Controller service
│   ├── agent-registry/           # Agent Registry service
│   ├── agent-runtime/            # Base agent execution runtime
│   ├── k8s-operator/             # Kubernetes Operator
│   ├── rest-api/                 # REST API for web portal
│   ├── a2a-core/                 # A2A protocol types
│   ├── a2a-server/               # A2A server implementation
│   ├── a2a-client/               # A2A client implementation
│   ├── mcp-core/                 # MCP protocol types
│   ├── mcp-client/               # MCP client for agents
│   └── mcp-servers/              # Custom MCP servers
│       ├── stripe-mcp/
│       └── redis-mcp/
├── ui/                           # React + CopilotKit A2UI portal
│   ├── src/
│   │   ├── components/           # React components (A2UIRenderer wrapper)
│   │   ├── hooks/                # React hooks (useSchema)
│   │   ├── schemas/              # A2UI JSON schemas (auto-discovered)
│   │   │   ├── taskForm.json
│   │   │   ├── progressStepper.json
│   │   │   ├── agentCard.json
│   │   │   ├── testResultsPanel.json
│   │   │   ├── tcpGauge.json
│   │   │   └── schemaLoader.ts   # Vite import.meta.glob loader
│   │   ├── styles/               # CSS files
│   │   └── services/             # API clients (future)
│   └── package.json
├── helm/
│   └── forgemaster/              # Helm chart
├── docker/                       # Dockerfiles
├── ansible/                      # Infrastructure automation
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
| **Orchestration** | Kubernetes (Kind) |
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
| Demo failures | **Dry-run mode as backup**; prepare fallback scenarios |
| Backend not ready for demo | **Web portal works standalone with mock data** |

---

## MVP Scope (Hackathon)

**Must Have:**
- [ ] Web portal with dry-run mode (Phase 1)
- [ ] TCP Controller with PID logic
- [ ] Agent Registry with basic registration
- [ ] Orchestrator + Test Generator + Code Generator agents
- [ ] Single working demo: E-commerce API generation

**Nice to Have:**
- [ ] Domain template reuse
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

1. **Initialize Rust workspace** — Set up crate structure
2. **Create Kind cluster** — Ansible playbook for local K8s
3. **Build UI Portal** — React + Lit with dry-run mode
4. **Implement REST API** — Connect portal to backend
5. **Implement TCP Controller** — Core feedback loop logic
6. **Deploy first agent** — Test Generator as proof of concept
