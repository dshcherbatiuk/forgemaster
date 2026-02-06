## Agent Communication: A2A Protocol

Agents communicate using the **Agent2Agent (A2A) Protocol** — an open standard by Google (now Linux Foundation) for secure agent-to-agent communication.

### Why A2A?

| Protocol | Purpose |
|----------|---------|
| **MCP** | Agent ↔ Tools/Data communication |
| **A2A** | Agent ↔ Agent communication |

A2A complements MCP: use MCP for tools and data access, use A2A for inter-agent collaboration.

### A2A Core Concepts

```mermaid
flowchart LR
    subgraph ClientAgent["Client Agent"]
        CA[Orchestrator Agent]
    end
    
    subgraph Discovery["Discovery"]
        AC1[Agent Card]
        AC2[Agent Card]
        AC3[Agent Card]
    end
    
    subgraph RemoteAgents["Remote Agents (A2A Servers)"]
        RA1[Test Generator Agent]
        RA2[Code Generator Agent]
        RA3[Feedback Agent]
    end
    
    CA -->|"1. Discover"| Discovery
    CA -->|"2. Send Task"| RA1 & RA2 & RA3
    RA1 & RA2 & RA3 -->|"3. Return Artifacts"| CA
```

### Agent Card

Each agent exposes an **Agent Card** — a JSON manifest describing its capabilities:

```json
{
  "name": "test-generator-agent",
  "description": "Generates Gherkin E2E tests from task descriptions",
  "version": "1.0.0",
  "endpoint": "https://agents.forgemaster.io/test-generator",
  "capabilities": {
    "streaming": true,
    "pushNotifications": true
  },
  "skills": [
    {
      "name": "generate-gherkin",
      "description": "Analyzes task and generates Gherkin feature files",
      "inputModes": ["text"],
      "outputModes": ["text", "file"]
    }
  ],
  "authentication": {
    "type": "oauth2",
    "authorizationUrl": "https://auth.forgemaster.io/oauth/authorize"
  }
}
```

### A2A Message Flow

```mermaid
sequenceDiagram
    participant OA as Orchestrator Agent<br/>(A2A Client)
    participant TGA as Test Generator Agent<br/>(A2A Server)
    participant CGA as Code Generator Agent<br/>(A2A Server)
    participant TRA as Test Runner Agent<br/>(A2A Server)
    participant FBA as Feedback Agent<br/>(A2A Server)

    Note over OA: Task received from TCP Controller
    
    OA->>TGA: POST /tasks (Create Task)
    Note right of TGA: {task_id, message: "Build REST API..."}
    TGA-->>OA: 202 Accepted {task_id, status: "working"}
    TGA->>TGA: Generate Gherkin tests
    TGA-->>OA: SSE: {status: "completed", artifacts: [feature.gherkin]}
    
    OA->>CGA: POST /tasks (Create Task)
    Note right of CGA: {task_id, message: "Implement API", context: [...]}
    CGA-->>OA: 202 Accepted
    CGA->>CGA: Generate code
    CGA-->>OA: SSE: {status: "completed", artifacts: [api.py]}
    
    OA->>TRA: POST /tasks (Run Tests)
    Note right of TRA: {artifacts: [feature.gherkin, api.py]}
    TRA-->>OA: 202 Accepted
    TRA->>TRA: Execute Gherkin tests
    TRA-->>OA: SSE: {status: "completed", results: {passed: 3, failed: 2}}
    
    OA->>FBA: POST /tasks (Analyze)
    Note right of FBA: {test_results, execution_metrics}
    FBA-->>OA: SSE: {error: 0.4, recommendations: [...]}
    
    Note over OA: Report back to TCP Controller
```

### A2A Task States

```mermaid
stateDiagram-v2
    [*] --> Submitted: Client sends task
    Submitted --> Working: Agent accepts
    Working --> Working: Agent sends updates (SSE)
    Working --> InputRequired: Agent needs more info
    InputRequired --> Working: Client provides input
    Working --> Completed: Task done
    Working --> Failed: Task failed
    Working --> Canceled: Client cancels
    Completed --> [*]
    Failed --> [*]
    Canceled --> [*]
```

### Agent CRD with A2A Configuration

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: test-generator-agent
  namespace: tasks
spec:
  type: test-generator
  
  # A2A Server Configuration
  a2a:
    enabled: true
    endpoint: "/a2a"
    port: 8080
    
    # Agent Card configuration
    agentCard:
      description: "Generates Gherkin E2E tests from task descriptions"
      skills:
        - name: generate-gherkin
          description: "Analyzes task and generates Gherkin feature files"
          inputModes: ["text"]
          outputModes: ["text", "file"]
      capabilities:
        streaming: true
        pushNotifications: true
        
    # Authentication
    authentication:
      type: oauth2
      secretRef:
        name: a2a-oauth-credentials
        
    # Rate limiting
    rateLimit:
      requestsPerMinute: 60
      
  model:
    provider: anthropic
    name: claude-sonnet-4-20250514
    
  # MCP servers for tools/data access
  mcpServers:
    - name: filesystem-mcp
    
  resources:
    limits:
      memory: "512Mi"
      cpu: "500m"
```

### A2A + MCP Integration

```mermaid
flowchart TB
    subgraph Agent["Code Generator Agent"]
        A2AS[A2A Server<br/>Agent-to-Agent]
        Core[Agent Core<br/>LLM + Logic]
        MCPC[MCP Client<br/>Agent-to-Tools]
    end
    
    subgraph OtherAgents["Other Agents"]
        OA[Orchestrator Agent]
        TRA[Test Runner Agent]
    end
    
    subgraph MCPServers["MCP Servers"]
        GH[GitHub MCP]
        FS[Filesystem MCP]
        DB[Database MCP]
    end
    
    OA <-->|"A2A Protocol"| A2AS
    TRA <-->|"A2A Protocol"| A2AS
    A2AS <--> Core
    Core <--> MCPC
    MCPC <-->|"MCP Protocol"| GH & FS & DB
```

### K8s Service for A2A

```yaml
apiVersion: v1
kind: Service
metadata:
  name: test-generator-agent-a2a
  namespace: tasks
  labels:
    forgemaster.io/agent: test-generator-agent
    forgemaster.io/protocol: a2a
spec:
  selector:
    forgemaster.io/agent: test-generator-agent
  ports:
    - name: a2a
      port: 8080
      targetPort: 8080
      protocol: TCP
  type: ClusterIP
---
# Ingress for external A2A access
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: a2a-gateway
  namespace: tasks
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  rules:
    - host: agents.forgemaster.io
      http:
        paths:
          - path: /test-generator/.well-known/agent.json
            pathType: Prefix
            backend:
              service:
                name: test-generator-agent-a2a
                port:
                  number: 8080
          - path: /test-generator/
            pathType: Prefix
            backend:
              service:
                name: test-generator-agent-a2a
                port:
                  number: 8080
```
