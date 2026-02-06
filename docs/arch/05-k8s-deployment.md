## K8s Deployment Model

### Custom Resource Definitions (CRDs)

```mermaid
flowchart TB
    subgraph CRDs["Custom Resource Definitions"]
        DOM[Domain CRD]
        AT[AgentTask CRD]
        AG[Agent CRD]
        MCP[MCPServer CRD]
    end

    subgraph Operator["forgemaster-operator Deployment"]
        ATC[AgentTask Controller]
        AGC[Agent Controller]
        MCPC[MCPServer Controller]
    end

    subgraph Resources["Managed Resources"]
        NS[Namespaces]
        Pods[Pods]
        SVC[Services]
        CM[ConfigMaps]
        SEC[Secrets]
    end

    DOM -.->|templates| ATC
    AT --> ATC
    AG --> AGC
    MCP --> MCPC

    ATC --> NS & CM
    ATC -->|creates| AG & MCP
    AGC --> Pods & CM
    MCPC --> Pods & SVC & SEC
```

> **Note:** Controllers run as a single Operator Deployment, not as separate CRDs. Domain is cluster-scoped configuration managed by administrators.

### Domain CRD

Defines a domain — a specialized area of expertise with proven agent combinations, MCP servers, and task patterns.

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Domain
metadata:
  name: web-development
  namespace: forgemaster-system
spec:
  # Domain identification
  displayName: "Web Development"
  description: "Full-stack web applications, REST APIs, frontend frameworks"

  # Skills this domain provides
  skills:
    - rest-api
    - graphql
    - authentication
    - database-integration
    - frontend-spa
    - payment-processing

  # Task patterns this domain can handle (regex patterns)
  taskPatterns:
    - ".*REST API.*"
    - ".*web (app|application).*"
    - ".*frontend.*"
    - ".*e-commerce.*"
    - ".*checkout.*"

  # Agent templates available in this domain
  agentTemplates:
    domainSpecific:
      - name: api-architect
        type: code-generator
        systemPrompt: |
          You are an API architect specializing in REST and GraphQL APIs.
          Design clean, scalable API structures.
        model:
          provider: anthropic
          name: claude-sonnet-4-20250514
          temperature: 0.7

      - name: stripe-integrator
        type: code-generator
        systemPrompt: |
          You are a payment integration specialist.
          Implement secure Stripe payment flows.
        model:
          provider: anthropic
          name: claude-sonnet-4-20250514
          temperature: 0.5

    # References to shared agents (defined separately)
    sharedAgentRefs:
      - test-generator
      - reviewer
      - doc-generator

  # MCP servers required by this domain
  mcpServers:
    - name: github-mcp
      required: true
    - name: postgres-mcp
      required: false
    - name: stripe-mcp
      required: false

  # Matching configuration
  matching:
    minMatchScore: 0.75
    keywordWeights:
      "REST": 0.9
      "API": 0.8
      "web": 0.7
      "frontend": 0.8
      "Stripe": 0.9
      "checkout": 0.85

  # Domain composition rules
  composition:
    canComposeWith:
      - data-engineering
      - testing-qa
    compositionPatterns:
      - pattern: ".*real-time.*analytics.*"
        composeDomains: ["web-development", "data-engineering"]

status:
  phase: Active  # Active | Deprecated | Disabled
  registeredAgents: 5
  activeTasks: 3
  successRate: 0.87
  lastUsed: "2026-01-19T14:00:00Z"
```

### AgentTask CRD

Top-level resource that defines a task to be executed.

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
metadata:
  name: ecommerce-backend
  namespace: task-ecommerce-abc123
spec:
  description: |
    Create an e-commerce backend with product catalog, shopping cart,
    and checkout flow. Include Stripe integration.

  # Domain reference (auto-selected or user-specified)
  domain:
    name: web-development
    autoSelected: true
    matchScore: 0.94
    reason: "Task mentions e-commerce, checkout, Stripe — matched web-development domain"

  # TCP Controller settings
  controller:
    taskWeight: 1.0      # T coefficient
    contextWeight: 0.5   # C coefficient
    predictionWeight: 0.3 # P coefficient
    errorThreshold: 0.2
    maxIterations: 10

  # Test generation config
  testGenerator:
    format: gherkin
    framework: behave

  # Resource limits for spawned agents
  resourceQuota:
    maxAgents: 5
    maxMemory: "4Gi"
    maxCPU: "4"

status:
  phase: Running # Pending | Running | Succeeded | Failed
  iteration: 3
  currentError: 0.25
  testsTotal: 12
  testsPassed: 9
  agents:
    - name: api-architect-abc123
      status: Completed
    - name: stripe-integrator-def456
      status: Running
    - name: database-designer-ghi789
      status: Completed
```

### Agent CRD

Defines an individual agent instance.

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: api-architect-abc123
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
    forgemaster.io/type: code-generator
    forgemaster.io/domain: web-development
spec:
  type: code-generator

  # LLM configuration
  model:
    provider: anthropic
    name: claude-sonnet-4-20250514
    temperature: 0.7
    maxTokens: 4096

  # Agent prompt/instructions
  systemPrompt: |
    You are an API Architect agent specialized in REST APIs for e-commerce.
    Design and implement endpoints for product catalog, shopping cart, and checkout.
    Follow RESTful conventions and ensure proper error handling.

  # MCP servers this agent needs
  mcpServers:
    - name: github-mcp
    - name: postgres-mcp

  # Resource limits
  resources:
    limits:
      memory: "512Mi"
      cpu: "500m"
    requests:
      memory: "256Mi"
      cpu: "250m"

status:
  phase: Completed
  startTime: "2026-01-19T12:00:00Z"
  tokensUsed: 4521
  output:
    configMapRef: api-architect-abc123-output
```

### MCPServer CRD

Defines an MCP server instance.

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: MCPServer
metadata:
  name: stripe-mcp
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
spec:
  type: stripe

  # MCP server image
  image: forgemaster/mcp-stripe:latest

  # Server configuration
  config:
    capabilities:
      - payment_intents
      - customers
      - webhooks

  # Credentials reference
  credentialsSecret:
    name: stripe-credentials

  # Resource limits
  resources:
    limits:
      memory: "256Mi"
      cpu: "250m"

status:
  phase: Running
  endpoint: "http://stripe-mcp:3000"
  tools:
    - create_payment_intent
    - confirm_payment
    - create_customer
    - handle_webhook
```

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

### Test Runner Agent

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: test-runner-agent
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
    forgemaster.io/type: test-runner
spec:
  type: test-runner

  model:
    provider: anthropic
    name: claude-sonnet-4-20250514
    temperature: 0.3
    maxTokens: 2048

  systemPrompt: |
    You are a Test Runner agent. Your job is to:
    1. Execute Gherkin test scenarios for e-commerce backend
    2. Test product catalog, shopping cart, and checkout endpoints
    3. Verify Stripe payment integration
    4. Report pass/fail for each scenario with detailed failure messages

  # Test framework configuration
  testConfig:
    framework: behave
    timeout: 300s
    parallel: false

  mcpServers:
    - name: filesystem-mcp

  resources:
    limits:
      memory: "512Mi"
      cpu: "500m"

status:
  phase: Running
  lastExecution: "2026-01-19T12:05:00Z"
```

### Feedback Agent CRD

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: feedback-agent
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
    forgemaster.io/type: feedback
spec:
  type: feedback

  model:
    provider: anthropic
    name: claude-sonnet-4-20250514
    temperature: 0.5
    maxTokens: 2048

  systemPrompt: |
    You are a Feedback Collector agent. Your job is to:
    1. Analyze test results from Test Runner
    2. Calculate error metrics for e-commerce scenarios
    3. Identify failure patterns in checkout/payment flow
    4. Recommend adjustments for next iteration
    5. Update context memory with learnings

  # Metrics to collect
  metricsConfig:
    collectTokenUsage: true
    collectExecutionTime: true
    collectRetryCount: true
    analyzePatterns: true

  mcpServers:
    - name: prometheus-mcp

  resources:
    limits:
      memory: "256Mi"
      cpu: "250m"

status:
  phase: Running
  currentError: 0.25
  recommendations:
    - "Stripe Integrator missed payment failure edge case"
    - "Cart service needs quantity validation"
    - "Suggest adding retry logic for Stripe API timeouts"
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
            subgraph Operator["forgemaster-operator"]
                ATC[AgentTask Controller<br/>HTTP API + Reconciler]
                AGC[Agent Controller]
                MCPC[MCPServer Controller]
            end
        end

        subgraph DomainRegistry["Domain Registry"]
            DOM1[Domain: web-development]
            DOM2[Domain: data-engineering]
            DOM3[Domain: testing-qa]
            DOM4[Domain: ml-ai]
        end

        subgraph TaskNS["task-ecommerce-abc123 namespace"]
            subgraph Task1["AgentTask: ecommerce-backend"]
                direction TB
                DomainRef["Domain: web-development<br/>matchScore: 0.94"]
                subgraph CoreAgents["Core Agents (Shared)"]
                    TGA[Test Generator Agent]
                    OA[Orchestrator Agent]
                    FBA[Feedback Agent]
                    TRA[Test Runner Agent]
                end
                subgraph ExecutorAgents["Executor Agents (Domain-Specific)"]
                    AG1[API Architect Agent<br/>REST endpoints]
                    AG2[Stripe Integrator Agent<br/>payment flow]
                    AG3[Database Designer Agent<br/>product/cart schema]
                end
                subgraph MCPServers["MCP Servers"]
                    MCP1[github-mcp<br/>code repository]
                    MCP2[postgres-mcp<br/>database access]
                    MCP3[stripe-mcp<br/>payment API]
                end
                CM1["ConfigMap: ecommerce-tests<br/>12 Gherkin scenarios"]
            end
        end

        subgraph Storage["Storage"]
            Redis[(Redis<br/>Context Memory)]
        end
    end

    U -->|"submit task"| WEB
    WEB -->|"POST /tasks"| ATC
    ATC -->|"create AgentTask CR"| API
    ATC <-.->|"watch status"| API
    WEB <-.->|"WebSocket progress"| ATC

    TCPC -->|"error signal"| OA
    TCPC <-->|"read metrics"| FBA
    OA -->|"create/adjust agents"| API

    ATC -->|"reconcile"| Task1
    DOM1 -.->|"provides templates"| Task1
    AGC -->|"manages"| CoreAgents & ExecutorAgents
    MCPC -->|"manages"| MCPServers
    TGA -->|"creates"| CM1

    CoreAgents & ExecutorAgents --> Redis
```

**Task Flow:**

1. User submits: "Create an e-commerce backend with product catalog, shopping cart, and checkout flow. Include Stripe integration."
2. Domain Controller matches to `web-development` domain (score: 0.94)
3. AgentTask Controller creates isolated namespace `task-ecommerce-abc123`
4. Core agents spawn (shared across all domains)
5. Domain provides templates for: API Architect, Stripe Integrator, Database Designer
6. MCP servers provision: github-mcp, postgres-mcp, stripe-mcp
7. Test Generator creates 12 Gherkin scenarios for product CRUD, cart operations, checkout flow
8. TCP feedback loop runs until all tests pass

### Operator Reconciliation Loop

Example: E-commerce backend task flow.

```mermaid
sequenceDiagram
    actor U as User
    participant WEB as A2UI Web Portal
    participant ATC as AgentTask Controller
    participant API as K8s API
    participant TCPC as TCP Controller
    participant DOM as Domain CR
    participant AGC as Agent Controller
    participant MCPC as MCPServer Controller
    participant TGA as Test Generator Agent
    participant OA as Orchestrator Agent
    participant EX as Executor Agents
    participant MCP as MCP Servers
    participant TRA as Test Runner Agent
    participant FBA as Feedback Agent

    U->>WEB: Submit "Create e-commerce backend with Stripe"
    WEB->>ATC: POST /tasks {description}
    ATC->>API: Create AgentTask CR
    ATC->>API: Watch AgentTask status

    ATC->>DOM: Match task to domain
    ATC->>ATC: Calculate match scores
    Note over ATC: e-commerce, checkout, Stripe → web-development (0.94)
    ATC->>API: Update AgentTask with domain (web-development)

    ATC->>AGC: Create core agents (shared)
    AGC->>TGA: Spawn Test Generator
    AGC->>OA: Spawn Orchestrator
    AGC->>TRA: Spawn Test Runner
    AGC->>FBA: Spawn Feedback Agent

    TGA->>TGA: Analyze task, generate Gherkin tests
    Note over TGA: 12 scenarios: products, cart, checkout
    TGA->>API: Create tests ConfigMap

    OA->>DOM: Get domain agent templates
    DOM->>OA: Return: api-architect, stripe-integrator, db-designer
    OA->>API: Create Agent CRs
    OA->>API: Create MCPServer CRs
    API->>MCPC: MCPServer CRs created
    MCPC->>MCP: Spawn github-mcp
    MCPC->>MCP: Spawn postgres-mcp
    MCPC->>MCP: Spawn stripe-mcp
    MCP->>API: MCP endpoints ready
    AGC->>EX: Spawn executor agents

    loop TCP Feedback Loop (iterations 1-5)
        EX->>MCP: github-mcp: read/write code
        EX->>MCP: postgres-mcp: create schema
        EX->>MCP: stripe-mcp: setup payment flow
        Note over EX: Build product API, cart service, checkout flow
        EX->>API: Output ready
        TRA->>MCP: filesystem-mcp: read generated code
        TRA->>TRA: Run Gherkin tests
        TRA->>FBA: Test results (9/12 passed)
        FBA->>TCPC: Error signal (0.25)
        TCPC->>TCPC: PID calculation: error = 0.25, threshold = 0.2
        TCPC->>API: Update AgentTask status
        API->>ATC: Status change event
        ATC->>WEB: WebSocket progress update
        WEB->>U: Live progress (9/12 tests, iteration 3)
        alt error > threshold (0.25 > 0.2)
            TCPC->>OA: Adjustment signal
            FBA->>OA: "Stripe payment failure edge case missing"
            OA->>API: Update stripe-integrator prompt
            OA->>API: Request additional MCP if needed
        else error <= threshold
            TCPC->>API: Mark task Succeeded
            ATC->>DOM: Update domain success metrics
            MCPC->>MCP: Terminate MCP servers
            ATC->>WEB: Task completed
            WEB->>U: Task completed successfully
        end
    end
```

### Resource Management

- **Namespace per task** — isolation
- **HPA** — scale agents based on load
- **Pod lifecycle** — terminate on completion
- **MCP servers as sidecars** — co-located with agents
- **Domain registry** — cluster-wide shared resource

### Helm Chart Deployment

All CRDs and controllers are deployed via Helm charts. See [10-helm-charts.md](10-helm-charts.md) for full details.

```mermaid
flowchart TB
    subgraph HelmCharts["Helm Charts"]
        MC[forgemaster-crds]
        MO[forgemaster-operator]
        MD[forgemaster-domains]
    end

    subgraph Deployed["Deployed Resources"]
        CRDs[Custom Resource Definitions]
        Controllers[Controllers & Operators]
        Domains[Domain Registry]
    end

    MC -->|"helm install"| CRDs
    MO -->|"helm install"| Controllers
    MD -->|"helm install"| Domains
```

**Quick Start:**

```bash
# Add Helm repository
helm repo add forgemaster https://charts.forgemaster.io

# Install CRDs first
helm install forgemaster-crds forgemaster/forgemaster-crds

# Install operators
helm install forgemaster-operator forgemaster/forgemaster-operator \
  --namespace forgemaster-system \
  --create-namespace

# Install default domains
helm install forgemaster-domains forgemaster/forgemaster-domains \
  --namespace forgemaster-system
```

**Custom Domain Installation:**

```bash
# Install with custom domain values
helm install forgemaster-domains forgemaster/forgemaster-domains \
  --namespace forgemaster-system \
  -f custom-domains.yaml
```

Example `custom-domains.yaml`:

```yaml
domains:
  - name: fintech
    displayName: "FinTech Development"
    skills:
      - payment-processing
      - fraud-detection
      - regulatory-compliance
    agentTemplates:
      - name: compliance-checker
        type: reviewer
        systemPrompt: "You verify PCI-DSS and SOX compliance..."
```

---

