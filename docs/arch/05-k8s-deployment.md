## K8s Deployment Model

### Custom Resource Definitions (CRDs)

```mermaid
flowchart TB
    subgraph CRDs["Custom Resource Definitions"]
        AT[AgentTask CRD]
        AG[Agent CRD]
    end

    subgraph Controllers["Controllers"]
        ATC[AgentTask Controller]
        AGC["Agent Controller (MCP)"]
    end

    subgraph Resources["Managed Resources"]
        NS[Namespaces]
        Pods[Pods]
        RBAC[RBAC]
        SEC[Secrets]
    end

    AT --> ATC
    AT --> AGC
    AG --> AGC

    ATC --> NS
    AGC -->|creates orchestrator| AG
    AGC --> Pods & RBAC & SEC
```

Each controller runs as a separate Deployment:
- AgentTask Controller: [`crates/fm-controller-agenttask/helm/`](../../crates/fm-controller-agenttask/helm/)
- Agent Controller: [`crates/fm-controller-agent/helm/`](../../crates/fm-controller-agent/helm/)

MCP servers are external services — agents connect to them directly via `spec.mcpServers` references. No dedicated MCP controller is needed.

### AgentTask CRD

Top-level resource that defines a task to be executed. See [`crates/fm-controller-agenttask/helm/templates/agenttask-crd.yaml`](../../crates/fm-controller-agenttask/helm/templates/agenttask-crd.yaml) for the full CRD definition.

### Agent CRD

Defines an individual agent instance. See [`crates/fm-controller-agent/src/crd/agent/crd.rs`](../../crates/fm-controller-agent/src/crd/agent/crd.rs) for the full spec.

The Agent Controller creates pods with the appropriate runtime image (e.g. `fm-agent-runtime-claude`). Config is injected via env vars — see [`crates/fm-controller-agent/src/controller/pod_builder.rs`](../../crates/fm-controller-agent/src/controller/pod_builder.rs).

### Test Storage (ConfigMaps)

Test definitions are stored in ConfigMaps rather than a separate CRD. The Test Generator Agent creates these, and the Test Runner Agent executes them.

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: ecommerce-tests
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
    forgemaster.io/type: test-suite
data:
  format: gherkin
  framework: behave
  product-catalog.feature: |
    Feature: Product Catalog API
      Scenario: Create a new product
        Given the API is running
        When I POST to "/products" with valid data
        Then the response status is 201
  shopping-cart.feature: |
    Feature: Shopping Cart
      Scenario: Add item to cart
        Given a product exists
        When I POST to "/cart/items"
        Then the cart is updated
  checkout.feature: |
    Feature: Checkout with Stripe
      Scenario: Successful checkout
        Given I have items in cart
        When I POST to "/checkout"
        Then I receive order confirmation
```

### Cluster Architecture

Example: E-commerce backend task — "Create an e-commerce backend with product catalog, shopping cart, and checkout flow. Include Stripe integration."

```mermaid
flowchart TB
    U[User]
    WEB[A2UI Web Portal]

    subgraph Cluster["Kubernetes Cluster"]
        subgraph ControlPlane["forgemaster-system namespace"]
            API[K8s API]
            TCPC[TCP Controller<br/>PID feedback loop]
            ATC[AgentTask Controller<br/>WebSocket + Reconciler]
            AGC["Agent Controller (MCP)"]
        end

        subgraph TaskNS["task-ecommerce-abc123 namespace"]
            subgraph Task1["AgentTask: ecommerce-backend"]
                direction TB
                subgraph Agents["Agent Pods (fm-agent-runtime-claude)"]
                    OA[Orchestrator Agent]
                    TGA[Test Generator Agent]
                    AG1[Code Generator Agent]
                    AG2[Reviewer Agent]
                end
                CM1["ConfigMap: ecommerce-tests<br/>Gherkin scenarios"]
            end
        end
    end

    U -->|"submit task"| WEB
    WEB <-->|"WebSocket"| ATC
    ATC -->|"create AgentTask CR"| API
    ATC <-.->|"watch status"| API

    TCPC -->|"error signal"| OA
    AGC -->|"reconciles Agent CRs"| Agents

    ATC -->|"reconcile"| Task1
    OA <-->|"A2A"| AG1 & AG2 & TGA
    Agents -->|"MCP"| API
    Agents -->|"update own CR"| API
    TGA -->|"creates"| CM1
```

**Task Flow:**

1. User submits task via A2UI Web Portal
2. AgentTask Controller creates AgentTask CR and isolated namespace
3. Agent Controller detects new AgentTask (Running), creates Orchestrator Agent CR
4. Agent Controller reconciles Agent CR, creates pod with `fm-agent-runtime-claude`
5. Orchestrator pod sends initial prompt, stays alive for A2A/MCP communication
6. Orchestrator coordinates child agents via A2A, agents use MCP for tool access (filesystem, GitHub, K8s API)
7. Agents update their own CR status (phase, tokens used)
8. TCP feedback loop runs until error threshold is met

### Reconciliation Loop

Example: E-commerce backend task flow.

```mermaid
sequenceDiagram
    actor U as User
    participant WEB as A2UI Web Portal
    participant ATC as AgentTask Controller
    participant API as K8s API
    participant AGC as Agent Controller (MCP)
    participant OA as Orchestrator Agent
    participant Agents as Child Agents
    participant TCPC as TCP Controller

    U->>WEB: Submit task
    WEB->>ATC: WebSocket: submit_task {description}
    ATC->>API: Create AgentTask CR + namespace

    Note over ATC: Reconciler transitions: Pending → Running

    AGC-->>API: Watch detects new AgentTask CR (Running)
    AGC->>API: Create Orchestrator Agent CR
    AGC->>API: Create pod (fm-agent-runtime-claude)

    OA->>OA: Send initial prompt to Claude API
    Note over OA: Stays alive for A2A / MCP

    OA->>API: MCP: Create child Agent CRs (code-gen, test-gen, reviewer)
    AGC-->>API: Watch detects new Agent CRs
    AGC->>API: Create child agent pods

    Note over Agents: Each agent sends initial prompt, stays alive

    loop TCP Feedback Loop (until error ≤ threshold)
        OA->>Agents: Coordinate via A2A
        Agents->>API: MCP: use tools (filesystem, GitHub, K8s API)
        Agents->>API: Update own CR status (tokens, phase)

        ATC->>ATC: Reconciler detects status change
        ATC->>WEB: WebSocket: push status update
        WEB->>U: Live progress

        TCPC->>TCPC: PID calculation
        TCPC->>OA: Control signal (adjust/continue/complete)

        alt error > threshold
            OA->>API: MCP: create/update Agent CRs
        else error ≤ threshold
            OA->>API: MCP: mark task succeeded
            ATC->>WEB: Task completed
        end
    end
```

### Resource Management

- **Namespace per task** — isolation between tasks
- **Long-lived agent pods** — stay alive for A2A/MCP, exit on SIGTERM or task completion
- **RBAC per agent** — ServiceAccount + ClusterRoleBinding for K8s API access
- **Owner references** — agent pods are owned by Agent CR (garbage collected on deletion)


