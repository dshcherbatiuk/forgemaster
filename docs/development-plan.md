# ForgeMaster Development Plan

**Project:** ForgeMaster — Meta-Agent System for AgentForge Hackathon 2026

**Team:** CSM-101

**Last Updated:** 2026-02-07

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
│   [██████████]   [████████░░]   [░░░░░░░░░░]   [░░░░░░░░░░]     │
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
- [x] Apply AgentTask CRD to cluster

### 1.5 Helm Charts & Deployment

- [x] Create Helm chart for UI (ui/helm/)
- [x] Define values.yaml with configurable options
- [x] Create templates (deployment, service, gateway, httproute)
- [x] Create Dockerfile for UI
- [x] Create Ansible roles (gateway, ui, fm-controller-agenttask, fm-controller-agent, fm-agent-runtime-claude)
- [x] Create site.yml playbook
- [x] Introduce Ansible environments (environments/default/ with inventory, group_vars, local.env)
- [x] DRY: shared variables (image_tag, image_registry, build_enabled, image_pull_policy)
- [x] Makefile ENV support (`make cluster ENV=staging`)
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
- [x] Implement WebSocket server with `/ws` endpoint (see [WS Protocol](arch/17-ws-protocol.md))
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
- [x] Multi-task support with tab bar (ActiveTaskStore, configurable limit, combined schema per push)
- [x] Task limit enforcement with client error notification on exceed
- [x] Agent info display in task status card (fetches Agent CRs via AgentFetcher)
- [x] Fix A2UI theme crash: invalid usageHint "button" → "body" for tab bar Text components
- [ ] Implement clarification relay (agent → UI → agent)
- [ ] Implement A2UI schema diff (push only changes)
- [x] Add namespace lifecycle management (create namespace on Pending→Running, delete on task deletion with finalizer)

### 2.2 Agent Runtime (fm-agent-runtime-claude)

- [x] Create agent runtime crate for Anthropic Claude
- [x] Implement Claude API integration (SSE streaming via eventsource-stream)
- [x] Implement config loading from env vars (TASK_PROMPT, MODEL_NAME, etc.)
- [x] Add conversation management
- [x] Create Dockerfile (multi-stage build)
- [x] Create Ansible role (build-only, no Helm deploy — pods managed by Agent Controller)
- [x] Implement status updater (patch Agent CR status from runtime pod)
- [x] Long-lived runtime (sends initial prompt, stays alive for A2A/MCP)
- [x] Add MCP client integration (rmcp SDK, conversation loop with tool calling, CompositeToolExecutor)
- [x] Add multi-MCP-server support (McpServerRef with per-server port, MCP_SERVER_URLS injection)
- [x] Add configurable default MCP servers (DEFAULT_MCP_SERVERS env var, Ansible/Helm wiring)
- [ ] Add A2A communication support

### 2.3 Agent Controller (fm-controller-agent)

- [x] Define Agent CRD schema
- [x] Create Helm chart
- [x] Implement Agent CRD structs in Rust (leaf types + core types)
- [x] Implement controller skeleton (ReconcileStrategy, DashMap dispatcher, phase strategies)
- [x] Watch AgentTask CRs — create Orchestrator Agent CR when task enters Running
- [x] Implement dual watch via `tokio::select!` (Agent reconciler + AgentTask watcher)
- [x] Containerize (Dockerfile)
- [x] Implement pod_builder (builds runtime pods from Agent CRs with env vars, owner refs, resource limits)
- [x] Implement PendingStrategy (check/create pod, update status with podRef, transition to Running)
- [x] Implement RunningStrategy (watch pod phase, extract failure messages, transition to Succeeded/Failed)
- [x] Add runtime-agent config to Helm chart (image, LLM provider secret)
- [x] ControllerContext reads config from env vars (fail-fast)
- [x] Watch Agent CRs across all namespaces (Api::all)
- [x] Rename systemPrompt to taskPrompt in CRD and Helm chart
- [x] 3-phase orchestrator prompt (Requirements Analysis → Architecture → Orchestration)
- [x] Deploy to K8s
- [x] Implement RBAC propagator (ServiceAccount + ClusterRoleBinding per namespace)
- [x] Implement secret propagator (copy LLM provider secret to task namespace)
- [x] Wire LLM_PROVIDER_DEFAULT_MODEL through config chain (env → Ansible → Helm → ConfigMap → context)

### 2.4 Core Agents

#### Orchestrator Agent (per-task Agent CR, created by Agent Controller)

> **See:** [Requirements Preparation](arch/04c-requirements-preparation.md) for what the orchestrator must produce before provisioning agents.

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
- Agent Runtime crate with Claude API integration (fm-agent-runtime-claude)
- Agent Controller managing Agent CRs and pods
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

### 3.2 MCP Protocol Implementation

- [x] Implement MCP client for agent runtime (rmcp SDK — see [ADR-0006](adr/0006-mcp-client-sdk-selection.md))
- [x] Add tool calling conversation loop (multi-turn tool_use/tool_result cycling)
- [x] Add CompositeToolExecutor for multiple MCP server aggregation
- [x] Add McpServerRef with per-server port and K8s DNS URL building
- [x] Wire MCP_SERVER_URLS injection in pod_builder
- [x] Add DEFAULT_MCP_SERVERS config (env → Ansible → Helm → ConfigMap)
- [ ] Implement Agent Controller MCP server (create_agent, list_agents, get_agent_status)
- [ ] Integrate external MCP servers (filesystem, GitHub, K8s API)
- [ ] Test tool discovery and execution end-to-end
- [ ] Dynamic MCP server registry (future — out of scope for now)

### 3.3 A2A Protocol Implementation

- [ ] Create a2a-core crate (types, traits)
- [ ] Implement A2A server (Axum-based)
- [ ] Implement A2A client
- [ ] Add Agent Card generation
- [ ] Implement SSE streaming for task updates
- [ ] Test agent-to-agent communication

**Deliverables:**
- TCP Controller running with PID feedback loop
- MCP client integrated into agent runtime
- A2A protocol working between agents
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
│   │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │   │
│   │  │AgentTask         │  │Agent Controller  │  │TCP Controller│  │   │
│   │  │Controller        │  │  (MCP)           │  │  (PID Math)  │  │   │
│   │  │(WebSocket + K8s) │  │                  │  └──────────────┘  │   │
│   │  │+ Schema Cache    │  │                  │                     │   │
│   │  └──────────────────┘  └──────────────────┘                     │   │
│   └─────────────────────────────┬───────────────────────────────────┘   │
│                                 │ A2A + MCP                              │
│   ┌─────────────────────────────▼───────────────────────────────────┐   │
│   │                    TASK NAMESPACE (task-xxxxx)                   │   │
│   │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │   │
│   │  │Orchestrator│ │Test Gen    │ │Code Gen    │ │Reviewer    │   │   │
│   │  │   Agent    │ │  Agent     │ │  Agent     │ │   Agent    │   │   │
│   │  └─────┬──────┘ └────────────┘ └────────────┘ └────────────┘   │   │
│   │        │ A2A          ▲               ▲              ▲          │   │
│   │        └──────────────┴───────────────┴──────────────┘          │   │
│   │                       │ MCP (K8s API, filesystem, GitHub)       │   │
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
├── Cargo.toml                        # Workspace root
├── crates/
│   ├── fm-controller-agenttask/      # AgentTask CRD + Controller
│   │   ├── helm/
│   │   └── src/
│   │
│   ├── fm-controller-agent/          # Agent CRD + Controller (MCP)
│   │   ├── helm/
│   │   └── src/
│   │
│   ├── fm-agent-runtime-claude/      # Agent runtime — Anthropic Claude
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── mcp_client/           # rmcp SDK client (connect, list_tools, call_tool)
│   │       ├── tool_executor/        # ToolExecutor trait, CompositeToolExecutor, NoOp
│   │       └── conversation_loop.rs  # Multi-turn tool calling loop
│   │
│   ├── fm-tcp-controller/            # TCP Controller service (future)
│   │
│   └── fm-a2a/                       # A2A protocol (future)
│
├── ui/                               # React + CopilotKit A2UI portal
│   ├── helm/
│   └── src/
│
├── ansible/
│   ├── site.yml
│   ├── environments/
│   │   └── default/                  # Local OrbStack environment
│   │       ├── inventory
│   │       ├── group_vars/all.yml
│   │       └── local.env
│   └── roles/
│       ├── gateway/
│       ├── ui/
│       ├── fm-controller-agenttask/
│       ├── fm-controller-agent/
│       └── fm-agent-runtime-claude/
│
└── docs/
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
| K8s complexity | OrbStack locally; use Helm for repeatability |
| Claude API rate limits | Implement retry with backoff; cache responses |
| A2A/A2UI spec changes | Pin versions; abstract behind interfaces |
| Time constraints | Prioritize MVP; cut features if needed |
| Demo failures | Prepare fallback scenarios; record backup video |

---

## MVP Scope (Hackathon)

**Must Have:**
- [x] Web portal with live task status (Phase 1)
- [x] Agent Controller with pod lifecycle (Phase 2)
- [x] Agent Runtime with Claude API (Phase 2)
- [x] MCP client integration (rmcp SDK, multi-server, tool calling loop)
- [ ] A2A protocol for agent coordination
- [ ] TCP Controller with PID logic
- [ ] Orchestrator + Test Generator + Code Generator agents
- [ ] Single working demo: E-commerce API generation

**Nice to Have:**
- [ ] Outcome validation
- [ ] Full A2UI component catalog

**Out of Scope (Post-Hackathon):**
- [ ] Multiple runtime providers (Gemini, ChatGPT, custom)
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
5. ~~**Agent Controller** — Watch Agent CRs, create pods, phase strategies~~ ✅
6. ~~**Agent Runtime (Claude)** — Claude API client with SSE streaming, long-lived runtime~~ ✅
7. ~~**Ansible Environments** — DRY config, local.env for secrets~~ ✅
8. ~~**Secret/RBAC propagation** — Copy LLM provider secret + RBAC to task namespaces~~ ✅
9. ~~**MCP client** — rmcp SDK, tool calling loop, CompositeToolExecutor, multi-server support~~ ✅
10. **Agent Controller MCP server** — Implement create_agent, list_agents, get_agent_status tools
11. **A2A protocol** — Agent-to-agent communication for orchestrator coordination
12. **TCP Controller** — PID feedback loop (needs agent test results)
