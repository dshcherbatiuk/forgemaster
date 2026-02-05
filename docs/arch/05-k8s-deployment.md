## K8s Deployment Model

### Custom Resource Definitions (CRDs)

```mermaid
flowchart TB
    subgraph CRDs["Custom Resource Definitions"]
        DOM[Domain CRD]
        AT[AgentTask CRD]
        AG[Agent CRD]
        MCP[MCPServer CRD]
        TS[TestSuite CRD]
    end

    subgraph Controllers["Custom Controllers"]
        DOMC[Domain Controller]
        ATC[AgentTask Controller]
        AGC[Agent Controller]
        MCPC[MCPServer Controller]
        TSC[TestSuite Controller]
    end

    subgraph Resources["Managed Resources"]
        Pods[Pods]
        SVC[Services]
        CM[ConfigMaps]
        SEC[Secrets]
    end

    DOM --> DOMC
    AT --> ATC
    AG --> AGC
    MCP --> MCPC
    TS --> TSC

    DOMC --> CM
    ATC --> Pods & SVC
    AGC --> Pods & CM
    MCPC --> Pods & SVC & SEC
    TSC --> Pods & CM
```

### Domain CRD

Defines a domain — a specialized area of expertise with proven agent combinations, MCP servers, and task patterns.

```yaml
apiVersion: metaagent.io/v1alpha1
kind: Domain
metadata:
  name: web-development
  namespace: metaagent-system
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
apiVersion: metaagent.io/v1alpha1
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
apiVersion: metaagent.io/v1alpha1
kind: Agent
metadata:
  name: api-architect-abc123
  namespace: task-ecommerce-abc123
  labels:
    metaagent.io/task: ecommerce-backend
    metaagent.io/type: code-generator
    metaagent.io/domain: web-development
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
apiVersion: metaagent.io/v1alpha1
kind: MCPServer
metadata:
  name: stripe-mcp
  namespace: task-ecommerce-abc123
  labels:
    metaagent.io/task: ecommerce-backend
spec:
  type: stripe

  # MCP server image
  image: metaagent/mcp-stripe:latest

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

### TestSuite CRD

Defines the Gherkin test suite (setpoint).

```yaml
apiVersion: metaagent.io/v1alpha1
kind: TestSuite
metadata:
  name: ecommerce-tests
  namespace: task-ecommerce-abc123
  labels:
    metaagent.io/task: ecommerce-backend
spec:
  format: gherkin
  framework: behave

  # Gherkin feature inline or from ConfigMap
  features:
    - name: product-catalog
      content: |
        Feature: Product Catalog API
          As an e-commerce client
          I want to manage products via REST API
          So that I can build a product catalog

          Scenario: Create a new product
            Given the API is running
            When I send a POST request to "/products" with body:
              """
              {
                "name": "Wireless Headphones",
                "price": 99.99,
                "category": "electronics"
              }
              """
            Then the response status should be 201
            And the response should contain "id"

          Scenario: List products by category
            Given products exist in category "electronics"
            When I send a GET request to "/products?category=electronics"
            Then the response status should be 200
            And the response should contain a list of products

    - name: shopping-cart
      content: |
        Feature: Shopping Cart
          As a customer
          I want to manage my shopping cart
          So that I can purchase multiple items

          Scenario: Add item to cart
            Given a product exists with id "prod-123"
            And I have an active cart session
            When I send a POST request to "/cart/items" with body:
              """
              {
                "productId": "prod-123",
                "quantity": 2
              }
              """
            Then the response status should be 200
            And the cart total should be updated

    - name: checkout-flow
      content: |
        Feature: Checkout with Stripe
          As a customer
          I want to complete checkout with Stripe payment
          So that I can purchase my cart items

          Scenario: Successful checkout
            Given I have items in my cart
            And I have a valid Stripe payment method
            When I send a POST request to "/checkout" with payment details
            Then the response status should be 200
            And I should receive an order confirmation
            And Stripe should have processed the payment

          Scenario: Payment failure handling
            Given I have items in my cart
            And I have an invalid payment method
            When I send a POST request to "/checkout" with payment details
            Then the response status should be 402
            And the response should contain "payment_failed"

  # Test Runner Agent reference
  runnerAgent:
    name: test-runner-agent

status:
  phase: Running
  lastRun: "2026-01-19T12:05:00Z"
  results:
    total: 12
    passed: 9
    failed: 3
    error: 0.25
  failedScenarios:
    - "Payment failure handling"
    - "List products by category"
    - "Add item to cart"
```

### Test Runner Agent CRD

```yaml
apiVersion: metaagent.io/v1alpha1
kind: Agent
metadata:
  name: test-runner-agent
  namespace: task-ecommerce-abc123
  labels:
    metaagent.io/task: ecommerce-backend
    metaagent.io/type: test-runner
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
apiVersion: metaagent.io/v1alpha1
kind: Agent
metadata:
  name: feedback-agent
  namespace: task-ecommerce-abc123
  labels:
    metaagent.io/task: ecommerce-backend
    metaagent.io/type: feedback
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
        subgraph ControlPlane["metaagent-system namespace"]
            REST[REST API Service]
            API[K8s API]
            TCPC[TCP Controller<br/>PID feedback loop]
            DOMC[Domain Controller]
            ATC[AgentTask Controller]
            AGC[Agent Controller]
            MCPC[MCPServer Controller]
            TSC[TestSuite Controller]
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
                TS1["TestSuite: ecommerce-tests<br/>12 Gherkin scenarios"]
            end
        end

        subgraph Storage["Storage"]
            Redis[(Redis<br/>Context Memory)]
        end
    end

    U -->|"submit task"| WEB
    WEB -->|"POST /tasks"| REST
    REST -->|"create AgentTask CR"| API
    API -->|"watch events"| ATC
    REST <-.->|"watch status"| API
    WEB <-.->|"SSE progress"| REST

    TCPC -->|"error signal"| OA
    TCPC <-->|"read metrics"| FBA
    OA -->|"create/adjust agents"| API

    DOMC -->|"manages"| DomainRegistry
    ATC -->|"reconcile"| Task1
    DOM1 -.->|"provides templates"| Task1
    AGC -->|"manages"| CoreAgents & ExecutorAgents
    MCPC -->|"manages"| MCPServers
    TSC -->|"manages"| TS1

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
    participant REST as REST API Service
    participant API as K8s API
    participant TCPC as TCP Controller
    participant DOMC as Domain Controller
    participant ATC as AgentTask Controller
    participant AGC as Agent Controller
    participant MCPC as MCPServer Controller
    participant TGA as Test Generator Agent
    participant OA as Orchestrator Agent
    participant EX as Executor Agents
    participant MCP as MCP Servers
    participant TRA as Test Runner Agent
    participant FBA as Feedback Agent

    U->>WEB: Submit "Create e-commerce backend with Stripe"
    WEB->>REST: POST /tasks {description}
    REST->>API: Create AgentTask CR
    API->>ATC: AgentTask created
    REST->>API: Watch AgentTask status

    ATC->>DOMC: Match task to domain
    DOMC->>DOMC: Calculate match scores
    Note over DOMC: e-commerce, checkout, Stripe → web-development (0.94)
    DOMC->>API: Assign domain (web-development)

    ATC->>AGC: Create core agents (shared)
    AGC->>TGA: Spawn Test Generator
    AGC->>OA: Spawn Orchestrator
    AGC->>TRA: Spawn Test Runner
    AGC->>FBA: Spawn Feedback Agent

    TGA->>TGA: Analyze task, generate Gherkin tests
    Note over TGA: 12 scenarios: products, cart, checkout
    TGA->>API: Create TestSuite CR

    OA->>DOMC: Get domain agent templates
    DOMC->>OA: Return: api-architect, stripe-integrator, db-designer
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
        API->>REST: Status change event
        REST->>WEB: SSE progress update
        WEB->>U: Live progress (9/12 tests, iteration 3)
        alt error > threshold (0.25 > 0.2)
            TCPC->>OA: Adjustment signal
            FBA->>OA: "Stripe payment failure edge case missing"
            OA->>API: Update stripe-integrator prompt
            OA->>API: Request additional MCP if needed
        else error <= threshold
            TCPC->>API: Mark task Succeeded
            DOMC->>DOMC: Update domain success metrics
            MCPC->>MCP: Terminate MCP servers
            REST->>WEB: Task completed
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
        MC[metaagent-crds]
        MO[metaagent-operator]
        MD[metaagent-domains]
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
helm repo add metaagent https://charts.metaagent.io

# Install CRDs first
helm install metaagent-crds metaagent/metaagent-crds

# Install operators
helm install metaagent-operator metaagent/metaagent-operator \
  --namespace metaagent-system \
  --create-namespace

# Install default domains
helm install metaagent-domains metaagent/metaagent-domains \
  --namespace metaagent-system
```

**Custom Domain Installation:**

```bash
# Install with custom domain values
helm install metaagent-domains metaagent/metaagent-domains \
  --namespace metaagent-system \
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

