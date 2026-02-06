# ADR-0005: Kubernetes-Native Architecture with CRDs

**Status:** Accepted

**Date:** 2026-02-05

**Decision Makers:** CSM-101

**Technical Area:** System Architecture, Infrastructure, Orchestration

## Context

ForgeMaster is a meta-agent system that dynamically creates, orchestrates, and manages task-specific agent subsystems. The system needs to:

1. **Spawn agents on-demand** — Create agent instances based on task requirements
2. **Manage agent lifecycle** — Start, monitor, restart, terminate agents
3. **Isolate tasks** — Prevent interference between concurrent tasks
4. **Scale dynamically** — Handle varying load and agent counts
5. **Persist state** — Track task progress, agent status, iteration history
6. **Recover from failures** — Handle crashes, restarts, partial failures
7. **Provide observability** — Monitor, log, and debug agent behavior

## Decision Drivers

- **Dynamic workloads** — Agent count varies per task (2-10+ agents)
- **Isolation requirements** — Tasks must not interfere with each other
- **Reliability** — System must recover from agent/controller failures
- **Scalability** — Support multiple concurrent tasks
- **Operational simplicity** — Minimize custom infrastructure code
- **Hackathon timeline** — Leverage existing primitives vs. building from scratch

## Considered Options

### Option 1: Custom Process Manager

Build a custom process manager that spawns agents as local processes.

**Pros:**
- Simple to start
- No infrastructure dependencies
- Full control over implementation

**Cons:**
- Must implement: process lifecycle, health checks, restart logic
- Must implement: resource limits, isolation
- Must implement: state persistence, recovery
- Must implement: scaling, load balancing
- Single point of failure (manager process)
- No built-in observability
- Significant code to maintain

### Option 2: Docker Compose / Swarm

Use Docker Compose for local development, Docker Swarm for production.

**Pros:**
- Familiar tooling
- Good for local development
- Container isolation

**Cons:**
- Limited orchestration capabilities
- No native CRD-like abstractions
- Swarm has limited adoption/support
- Manual service discovery
- Limited auto-scaling

### Option 3: Serverless (AWS Lambda, Cloud Run)

Deploy agents as serverless functions.

**Pros:**
- Auto-scaling built-in
- Pay-per-use pricing
- No infrastructure management

**Cons:**
- Cold start latency (bad for agent responsiveness)
- Execution time limits (Lambda: 15min max)
- Stateless by design (agents need context)
- Vendor lock-in
- Complex local development
- Cost unpredictable for long-running agents

### Option 4: Kubernetes with Custom Resource Definitions (CRDs)

Use Kubernetes as the orchestration platform with CRDs to model system concepts.

**Pros:**
- **Declarative model** — Describe desired state, K8s reconciles
- **Built-in primitives** — Pods, Services, ConfigMaps, Secrets
- **Isolation** — Namespaces provide task isolation
- **Self-healing** — Automatic restart on failure
- **Resource management** — CPU/memory limits, quotas
- **Scaling** — HPA, pod autoscaling
- **Observability** — Metrics, logs, events built-in
- **CRDs** — Model system concepts (AgentTask, Agent, MCPServer)
- **Controller pattern** — Proven reconciliation loop pattern
- **Ecosystem** — Helm, Prometheus, Grafana, etc.
- **Local development** — Kind, OrbStack, Minikube

**Cons:**
- Learning curve for K8s concepts
- CRD development requires kube-rs knowledge
- Overhead for simple use cases
- Requires cluster setup

### Option 5: Nomad

Use HashiCorp Nomad for workload orchestration.

**Pros:**
- Simpler than Kubernetes
- Multi-runtime (containers, VMs, binaries)
- Good HashiCorp ecosystem integration

**Cons:**
- Smaller community than K8s
- No native CRD equivalent
- Less tooling/ecosystem
- Would need custom abstractions

## Decision

**We choose Option 4: Kubernetes with Custom Resource Definitions (CRDs)**.

Kubernetes provides the exact primitives ForgeMaster needs:

### 1. CRDs Model System Concepts Naturally

```yaml
# AgentTask — maps directly to user's task request
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
metadata:
  name: ecommerce-backend
spec:
  description: "Create e-commerce API with Stripe"
status:
  phase: Running
  iteration: 3
  currentError: 0.25
```

The K8s API becomes our task API — create an AgentTask CR, and the system handles the rest.

### 2. Controller Pattern Matches TCP Feedback Loop

```
┌─────────────────────────────────────────────────┐
│           K8s Controller Pattern                 │
│                                                  │
│   Observe → Analyze → Act → Observe → ...       │
│      ↓         ↓        ↓                       │
│   Watch CR   Compare   Update                   │
│   status     desired   resources                │
│              vs actual                          │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│           TCP Controller Pattern                 │
│                                                  │
│   Measure → Calculate → Adjust → Measure → ... │
│      ↓          ↓          ↓                    │
│   Error      PID math    Swap/add              │
│   signal                 agents                 │
└─────────────────────────────────────────────────┘
```

The K8s reconciliation loop naturally implements our TCP feedback loop.

### 3. Namespace Isolation Per Task

```
Cluster
├── forgemaster-system/          # Control plane
│   ├── api-gateway
│   ├── tcp-controller
│   └── agent-registry
├── task-ecommerce-abc123/       # Task 1 (isolated)
│   ├── orchestrator-agent
│   ├── code-generator-agent
│   └── github-mcp
└── task-dataflow-def456/        # Task 2 (isolated)
    ├── orchestrator-agent
    ├── etl-agent
    └── postgres-mcp
```

### 4. Built-in Reliability

| Feature | K8s Provides | Custom Would Require |
|---------|--------------|---------------------|
| Process restart | Pod restartPolicy | Supervisor logic |
| Health checks | Liveness/readiness probes | Health endpoint + monitor |
| Resource limits | Pod resources | cgroups/ulimit management |
| Scaling | HPA | Custom autoscaler |
| Service discovery | DNS/Services | Service registry |
| Config management | ConfigMaps/Secrets | Config service |
| State persistence | etcd (CRs) | Database + ORM |
| Event streaming | Watch API | Event bus |

### 5. Operational Benefits

- **GitOps** — CRs can be version-controlled
- **Rollback** — K8s handles deployment rollbacks
- **Audit** — K8s events provide audit trail
- **RBAC** — Fine-grained access control
- **Multi-tenancy** — Namespace-based isolation

## Consequences

### Positive

- **Leverage K8s ecosystem** — Helm, Prometheus, Grafana, ArgoCD
- **Declarative operations** — Define desired state, K8s reconciles
- **Self-healing** — Automatic recovery from failures
- **Scalability** — Proven at massive scale
- **Portability** — Runs on any K8s (cloud, on-prem, local)
- **Reduced code** — Don't implement orchestration primitives
- **Standard patterns** — Controller pattern well-documented

### Negative

- **K8s dependency** — Requires K8s cluster to run
- **Learning curve** — Team needs K8s and kube-rs knowledge
- **Complexity** — K8s has many concepts to understand
- **Local setup** — Need Kind/OrbStack for development

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| K8s learning curve slows development | Medium | Medium | Use kube-rs examples, follow patterns |
| CRD design mistakes | Medium | High | Start simple, iterate based on needs |
| Local K8s performance issues | Low | Low | OrbStack is fast; optimize if needed |
| Over-engineering for hackathon | Low | Medium | Start with AgentTask + Agent CRDs only |

## Implementation Notes

### CRD Hierarchy

```
AgentTask (top-level)
├── creates → Namespace (task isolation)
├── creates → Agent CRs (orchestrator, generators, etc.)
├── creates → MCPServer CRs (tools)
└── creates → TestSuite CR (Gherkin tests)

AgentTask Controller watches AgentTask
Agent Controller watches Agent
MCPServer Controller watches MCPServer
```

### Tech Stack

| Component | Technology |
|-----------|------------|
| CRD definitions | kube-rs `#[derive(CustomResource)]` |
| Controllers | kube-rs runtime with `Controller::new()` |
| API Gateway | Axum + kube-rs client |
| Local cluster | OrbStack Kubernetes |
| Package manager | Helm |

### CRDs to Implement

| Priority | CRD | Purpose |
|----------|-----|---------|
| P0 | AgentTask | Top-level task resource |
| P0 | Agent | Agent instance |
| P1 | MCPServer | MCP server instance |
| P1 | TestSuite | Gherkin test suite |

### Controller Order

1. **AgentTask Controller** — Creates namespace, coordinates lifecycle
2. **Agent Controller** — Spawns/manages agent pods
3. **MCPServer Controller** — Spawns/manages MCP server pods
4. **TestSuite Controller** — Manages test execution

## Related

- [05-k8s-deployment.md](../arch/05-k8s-deployment.md) — Full K8s deployment architecture
- [ADR-0001: TCP Controller](0001-tcp-controller-vs-llm-agents.md) — TCP feedback loop design
- [Development Plan](../development-plan.md) — Implementation phases
