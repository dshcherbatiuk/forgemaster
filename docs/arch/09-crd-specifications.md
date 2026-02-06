# Custom Resource Definitions (CRDs)

ForgeMaster uses Kubernetes Custom Resource Definitions to model the domain. This document specifies the schema for each CRD.

## Overview

```mermaid
flowchart TB
    subgraph CRDs["Custom Resource Definitions"]
        AT[AgentTask]
        AG[Agent]
        MCP[MCPServer]
    end

    subgraph Hierarchy["Resource Hierarchy"]
        AT -->|creates| NS[Namespace]
        AT -->|owns| AG
        AT -->|owns| MCP
        AT -->|owns| CM[ConfigMaps]
    end
```

| CRD | API Group | Purpose | Scope |
|-----|-----------|---------|-------|
| AgentTask | `forgemaster.io/v1alpha1` | Top-level task resource | Namespaced |
| Agent | `forgemaster.io/v1alpha1` | Agent instance | Namespaced |
| MCPServer | `forgemaster.io/v1alpha1` | MCP server instance | Namespaced |

> **Note:** Test definitions are stored in ConfigMaps (created by Test Generator Agent), not as a separate CRD.

---

## CRD Descriptions

### AgentTask

**Purpose:** Top-level resource representing a user's task request.

**Created by:** AgentTask Controller (via WebSocket command) when user submits a task.

**Managed by:** AgentTask Controller — watches AgentTask CRs and reconciles state.

**Owns:** Namespace, Agent CRs, MCPServer CRs, ConfigMaps.

**Lifecycle:**

```mermaid
stateDiagram-v2
    [*] --> Pending: User submits task
    Pending --> Initializing: AgentTask Controller picks up
    Initializing --> Running: Namespace + Agents created
    Running --> Running: TCP iterations (error > threshold)
    Running --> Succeeded: error ≤ threshold
    Running --> Failed: max iterations or fatal error
    Succeeded --> [*]
    Failed --> [*]
```

**Key fields:**

| Field | Description |
|-------|-------------|
| `spec.description` | Natural language task description |
| `spec.controller` | TCP coefficients (T, C, P weights) |
| `status.phase` | Pending → Initializing → Running → Succeeded/Failed |
| `status.currentError` | Error signal from TCP Controller (0.0-1.0) |
| `status.iteration` | Current iteration number |
| `status.tests` | Test results (total, passed, failed) |

---

### Agent

**Purpose:** Individual agent instance that performs work using an LLM.

**Created by:** AgentTask Controller (core agents) or Orchestrator Agent (dynamic agents).

**Managed by:** Agent Controller — watches Agent CRs, creates Pods, manages lifecycle.

**Owned by:** AgentTask (garbage collected when task is deleted).

**Types:**

| Type | Role |
|------|------|
| `orchestrator` | Coordinates other agents, manages workflow |
| `code-generator` | Generates code based on requirements/tests |
| `test-generator` | Creates Gherkin test scenarios |
| `test-runner` | Executes tests and reports results |
| `feedback` | Analyzes results, calculates error signal |
| `reviewer` | Reviews code quality and suggests improvements |

**Lifecycle:**

```mermaid
stateDiagram-v2
    [*] --> Pending: Agent CR created
    Pending --> Running: Pod scheduled + started
    Running --> Running: Processing (LLM calls, MCP tools)
    Running --> Succeeded: Work completed
    Running --> Failed: Error or timeout
    Succeeded --> [*]: Output in ConfigMap
    Failed --> [*]: Error logged
```

**Key fields:**

| Field | Description |
|-------|-------------|
| `spec.type` | Agent type (orchestrator, code-generator, etc.) |
| `spec.model` | LLM configuration (provider, name, temperature) |
| `spec.systemPrompt` | Instructions for the agent |
| `spec.mcpServers` | List of MCP servers this agent can use |
| `status.phase` | Pending → Running → Succeeded/Failed |
| `status.tokensUsed` | Total tokens consumed |

---

### MCPServer

**Purpose:** MCP (Model Context Protocol) server that provides tools to agents.

**Created by:** AgentTask Controller based on task requirements.

**Managed by:** MCPServer Controller — watches MCPServer CRs, creates Pods and Services, manages lifecycle.

**Owned by:** AgentTask (garbage collected when task is deleted).

**Types:**

| Type | Tools Provided |
|------|----------------|
| `github` | read_file, write_file, create_branch, create_pr |
| `filesystem` | read, write, list, delete files |
| `postgres` | query, execute, schema operations |
| `stripe` | create_payment, refund, manage_customers |
| `redis` | get, set, delete, pub/sub |

**Lifecycle:**

```mermaid
stateDiagram-v2
    [*] --> Pending: MCPServer CR created
    Pending --> Running: Pod + Service created
    Running --> Running: Serving tool requests
    Running --> Terminated: AgentTask completed/deleted
    Terminated --> [*]: Garbage collected
```

**Key fields:**

| Field | Description |
|-------|-------------|
| `spec.type` | Server type (github, postgres, etc.) |
| `spec.image` | Container image for MCP server |
| `spec.credentialsSecret` | Secret reference for API keys |
| `status.phase` | Pending → Running → Failed |
| `status.endpoint` | Service URL for agents to connect |
| `status.tools` | List of available MCP tools |

---

## Resource Flow

```mermaid
sequenceDiagram
    actor User
    participant ATC as AgentTask Controller
    participant API as K8s API
    participant AG as Agents
    participant MCP as MCPServers

    User->>ATC: Submit task (WebSocket)
    ATC->>API: Create AgentTask CR
    ATC->>API: Create Agent CRs
    ATC->>API: Create MCPServer CRs
    AG->>MCP: Use tools (read/write files)
    AG->>API: Update status (iteration, error)
    API-->>ATC: Status changes (watch)
    ATC-->>User: Real-time progress (WebSocket)
```

---

## AgentTask CRD

Top-level resource representing a user's task request.

### Schema

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
metadata:
  name: string                    # Task identifier (e.g., "ecommerce-backend")
  namespace: string               # Task namespace (e.g., "task-abc123")
spec:
  # Task Description (required)
  description: string             # Natural language task description

  # TCP Controller Configuration
  controller:
    taskWeight: number            # T coefficient (default: 1.0)
    contextWeight: number         # C coefficient (default: 0.5)
    predictionWeight: number      # P coefficient (default: 0.3)
    errorThreshold: number        # Success threshold (default: 0.1)
    maxIterations: integer        # Max iterations before failure (default: 10)

  # Test Configuration
  testConfig:
    format: string                # "gherkin" | "pytest" | "jest"
    framework: string             # Test framework (e.g., "behave", "pytest")
    timeout: string               # Test timeout (e.g., "5m")

  # Resource Quotas
  resourceQuota:
    maxAgents: integer            # Max concurrent agents (default: 5)
    maxMemory: string             # Total memory limit (e.g., "4Gi")
    maxCPU: string                # Total CPU limit (e.g., "4")

  # Artifact Storage
  artifacts:
    repository: string            # Git repository URL
    branch: string                # Branch name (default: "main")
    path: string                  # Output path in repo

status:
  # Task Phase
  phase: string                   # Pending | Initializing | Running | Succeeded | Failed

  # Progress
  iteration: integer              # Current iteration number
  currentError: number            # Current error signal (0.0 - 1.0)
  startTime: string               # ISO 8601 timestamp
  completionTime: string          # ISO 8601 timestamp (if completed)

  # Test Results
  tests:
    total: integer                # Total test count
    passed: integer               # Passed tests
    failed: integer               # Failed tests
    skipped: integer              # Skipped tests

  # Agent Status
  agents:
    - name: string                # Agent name
      type: string                # Agent type
      phase: string               # Pending | Running | Succeeded | Failed

  # Conditions
  conditions:
    - type: string                # Condition type
      status: string              # True | False | Unknown
      reason: string              # Machine-readable reason
      message: string             # Human-readable message
      lastTransitionTime: string  # ISO 8601 timestamp
```

### Example

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
metadata:
  name: ecommerce-backend
  namespace: task-ecommerce-abc123
spec:
  description: |
    Create an e-commerce backend API with:
    - Product catalog with categories
    - Shopping cart management
    - Checkout with Stripe integration
    - Order history

  controller:
    taskWeight: 1.0
    contextWeight: 0.5
    predictionWeight: 0.3
    errorThreshold: 0.1
    maxIterations: 10

  testConfig:
    format: gherkin
    framework: behave
    timeout: 5m

  resourceQuota:
    maxAgents: 5
    maxMemory: 4Gi
    maxCPU: "4"

  artifacts:
    repository: https://github.com/user/ecommerce-api
    branch: main
    path: /src

status:
  phase: Running
  iteration: 3
  currentError: 0.25
  startTime: "2026-02-05T12:00:00Z"
  tests:
    total: 12
    passed: 9
    failed: 3
    skipped: 0
  agents:
    - name: orchestrator-abc123
      type: orchestrator
      phase: Running
    - name: code-generator-def456
      type: code-generator
      phase: Running
    - name: test-runner-ghi789
      type: test-runner
      phase: Succeeded
  conditions:
    - type: Initialized
      status: "True"
      reason: NamespaceCreated
      message: Task namespace created successfully
      lastTransitionTime: "2026-02-05T12:00:05Z"
    - type: AgentsReady
      status: "True"
      reason: AllAgentsRunning
      message: All required agents are running
      lastTransitionTime: "2026-02-05T12:00:30Z"
```

---

## Agent CRD

Represents an individual agent instance.

### Schema

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: string                    # Agent identifier
  namespace: string               # Task namespace
  labels:
    forgemaster.io/task: string   # Parent AgentTask name
    forgemaster.io/type: string   # Agent type

  ownerReferences:                # Garbage collection
    - apiVersion: forgemaster.io/v1alpha1
      kind: AgentTask
      name: string
      uid: string
spec:
  # Agent Type (required)
  type: string                    # orchestrator | code-generator | test-generator |
                                  # test-runner | feedback | reviewer

  # LLM Configuration (required)
  model:
    provider: string              # anthropic | openai | local
    name: string                  # Model name (e.g., "claude-sonnet-4-20250514")
    temperature: number           # Temperature (0.0 - 2.0)
    maxTokens: integer            # Max output tokens

  # System Prompt (required)
  systemPrompt: string            # Agent instructions

  # MCP Server References
  mcpServers:
    - name: string                # MCPServer CR name

  # A2A Configuration
  a2a:
    enabled: boolean              # Enable A2A server
    port: integer                 # A2A server port (default: 8080)

  # Resource Limits
  resources:
    limits:
      memory: string              # Memory limit (e.g., "512Mi")
      cpu: string                 # CPU limit (e.g., "500m")
    requests:
      memory: string              # Memory request
      cpu: string                 # CPU request

  # Environment Variables
  env:
    - name: string
      value: string
    - name: string
      valueFrom:
        secretKeyRef:
          name: string
          key: string

status:
  # Agent Phase
  phase: string                   # Pending | Running | Succeeded | Failed

  # Execution Info
  startTime: string               # ISO 8601 timestamp
  completionTime: string          # ISO 8601 timestamp
  tokensUsed: integer             # Total tokens consumed
  iterationsCompleted: integer    # Iterations this agent participated in

  # Pod Reference
  podRef:
    name: string                  # Pod name
    uid: string                   # Pod UID

  # Output Reference
  output:
    configMapRef: string          # ConfigMap with agent output

  # Conditions
  conditions:
    - type: string
      status: string
      reason: string
      message: string
      lastTransitionTime: string
```

### Example

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
metadata:
  name: code-generator-def456
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
    forgemaster.io/type: code-generator

  ownerReferences:
    - apiVersion: forgemaster.io/v1alpha1
      kind: AgentTask
      name: ecommerce-backend
      uid: abc123-def456-ghi789
spec:
  type: code-generator

  model:
    provider: anthropic
    name: claude-sonnet-4-20250514
    temperature: 0.7
    maxTokens: 4096

  systemPrompt: |
    You are a Code Generator agent specialized in Rust backend development.

    Your task:
    - Implement REST API endpoints based on test specifications
    - Use Axum framework for HTTP handling
    - Follow clean architecture principles
    - Include proper error handling

    Output format:
    - Generate complete, compilable Rust code
    - Include module structure
    - Add inline documentation

  mcpServers:
    - name: github-mcp
    - name: postgres-mcp

  a2a:
    enabled: true
    port: 8080

  resources:
    limits:
      memory: 512Mi
      cpu: 500m
    requests:
      memory: 256Mi
      cpu: 250m

  env:
    - name: RUST_LOG
      value: info
    - name: ANTHROPIC_API_KEY
      valueFrom:
        secretKeyRef:
          name: llm-credentials
          key: anthropic-api-key

status:
  phase: Running
  startTime: "2026-02-05T12:00:30Z"
  tokensUsed: 8542
  iterationsCompleted: 2
  podRef:
    name: code-generator-def456-pod
    uid: pod-uid-123
  conditions:
    - type: Ready
      status: "True"
      reason: PodRunning
      message: Agent pod is running and healthy
      lastTransitionTime: "2026-02-05T12:00:35Z"
```

---

## MCPServer CRD

Represents an MCP (Model Context Protocol) server instance.

### Schema

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: MCPServer
metadata:
  name: string                    # Server identifier
  namespace: string               # Task namespace
  labels:
    forgemaster.io/task: string   # Parent AgentTask name
    forgemaster.io/type: string   # Server type
  ownerReferences:
    - apiVersion: forgemaster.io/v1alpha1
      kind: AgentTask
      name: string
      uid: string
spec:
  # Server Type (required)
  type: string                    # github | postgres | filesystem | stripe | redis

  # Container Image
  image: string                   # Docker image (e.g., "forgemaster/mcp-github:latest")

  # Server Configuration
  config:
    # Type-specific configuration
    capabilities: []string        # Enabled capabilities
    settings: map[string]string   # Additional settings

  # Credentials
  credentialsSecret:
    name: string                  # Secret name
    keys:                         # Key mappings
      - secretKey: string         # Key in secret
        envVar: string            # Environment variable name

  # Resource Limits
  resources:
    limits:
      memory: string
      cpu: string
    requests:
      memory: string
      cpu: string

  # Health Check
  healthCheck:
    path: string                  # Health endpoint (default: "/health")
    port: integer                 # Health port (default: 3000)
    initialDelaySeconds: integer  # Startup delay
    periodSeconds: integer        # Check interval

status:
  # Server Phase
  phase: string                   # Pending | Running | Failed

  # Endpoint
  endpoint: string                # Service endpoint URL

  # Available Tools
  tools:
    - name: string                # Tool name
      description: string         # Tool description

  # Pod Reference
  podRef:
    name: string
    uid: string

  # Conditions
  conditions:
    - type: string
      status: string
      reason: string
      message: string
      lastTransitionTime: string
```

### Example

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: MCPServer
metadata:
  name: github-mcp
  namespace: task-ecommerce-abc123
  labels:
    forgemaster.io/task: ecommerce-backend
    forgemaster.io/type: github
  ownerReferences:
    - apiVersion: forgemaster.io/v1alpha1
      kind: AgentTask
      name: ecommerce-backend
      uid: abc123-def456-ghi789
spec:
  type: github

  image: forgemaster/mcp-github:latest

  config:
    capabilities:
      - read_file
      - write_file
      - create_branch
      - create_pull_request
      - list_files
    settings:
      defaultBranch: main
      autoCommit: "true"

  credentialsSecret:
    name: github-credentials
    keys:
      - secretKey: token
        envVar: GITHUB_TOKEN
      - secretKey: repository
        envVar: GITHUB_REPOSITORY

  resources:
    limits:
      memory: 256Mi
      cpu: 250m
    requests:
      memory: 128Mi
      cpu: 100m

  healthCheck:
    path: /health
    port: 3000
    initialDelaySeconds: 5
    periodSeconds: 10

status:
  phase: Running
  endpoint: http://github-mcp.task-ecommerce-abc123.svc.cluster.local:3000
  tools:
    - name: read_file
      description: Read file contents from repository
    - name: write_file
      description: Write or update file in repository
    - name: create_branch
      description: Create a new branch
    - name: create_pull_request
      description: Create a pull request
    - name: list_files
      description: List files in a directory
  podRef:
    name: github-mcp-pod
    uid: pod-uid-456
  conditions:
    - type: Ready
      status: "True"
      reason: ServerHealthy
      message: MCP server is responding to health checks
      lastTransitionTime: "2026-02-05T12:00:20Z"
```

---

## Generating CRDs with kube-rs

### Rust Struct Definitions

```rust
// crates/fm-core/src/crds/agent_task.rs

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "forgemaster.io",
    version = "v1alpha1",
    kind = "AgentTask",
    namespaced,
    status = "AgentTaskStatus",
    printcolumn = r#"{"name":"Phase", "type":"string", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Iteration", "type":"integer", "jsonPath":".status.iteration"}"#,
    printcolumn = r#"{"name":"Error", "type":"number", "jsonPath":".status.currentError"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskSpec {
    pub description: String,

    #[serde(default)]
    pub controller: ControllerConfig,

    #[serde(default)]
    pub test_config: TestConfig,

    #[serde(default)]
    pub resource_quota: ResourceQuota,

    #[serde(default)]
    pub artifacts: Option<ArtifactConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskStatus {
    #[serde(default)]
    pub phase: TaskPhase,

    #[serde(default)]
    pub iteration: i32,

    #[serde(default)]
    pub current_error: f64,

    pub start_time: Option<String>,
    pub completion_time: Option<String>,

    #[serde(default)]
    pub tests: TestResults,

    #[serde(default)]
    pub agents: Vec<AgentStatus>,

    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq)]
pub enum TaskPhase {
    #[default]
    Pending,
    Initializing,
    Running,
    Succeeded,
    Failed,
}

// Additional types...
```

### Generate CRD Manifests

```rust
// crates/fm-core/src/bin/gen_crds.rs

use fm_core::crds::{AgentTask, Agent, MCPServer};
use kube::CustomResourceExt;
use std::fs;

fn main() {
    let crds = vec![
        serde_yaml::to_string(&AgentTask::crd()).unwrap(),
        serde_yaml::to_string(&Agent::crd()).unwrap(),
        serde_yaml::to_string(&MCPServer::crd()).unwrap(),
    ];

    let output = crds.join("---\n");
    fs::write("manifests/crds.yaml", output).expect("Failed to write CRDs");

    println!("Generated CRDs to manifests/crds.yaml");
}
```

### Build and Generate

```bash
# Generate CRD manifests
cargo run --package fm-core --bin gen_crds

# Output: manifests/crds.yaml
```

---

## Applying CRDs to Cluster

### Manual Application

```bash
# Apply CRDs to cluster
kubectl apply -f manifests/crds.yaml

# Verify CRDs are installed
kubectl get crds | grep forgemaster

# Expected output:
# agenttasks.forgemaster.io      2026-02-05T12:00:00Z
# agents.forgemaster.io          2026-02-05T12:00:00Z
# mcpservers.forgemaster.io      2026-02-05T12:00:00Z
```

### Helm Chart

```yaml
# helm/forgemaster-crds/templates/crds.yaml
{{- range $path, $_ := .Files.Glob "crds/*.yaml" }}
{{ $.Files.Get $path }}
---
{{- end }}
```

```bash
# Install CRDs via Helm
helm install forgemaster-crds ./helm/forgemaster-crds

# Upgrade CRDs
helm upgrade forgemaster-crds ./helm/forgemaster-crds
```

### Makefile Target

```makefile
# Makefile

.PHONY: crds-generate crds-apply crds-delete

crds-generate:
	cargo run --package fm-core --bin gen_crds

crds-apply: crds-generate
	kubectl apply -f manifests/crds.yaml

crds-delete:
	kubectl delete -f manifests/crds.yaml --ignore-not-found
```

---

## Validation

### Verify CRD Installation

```bash
# Check CRDs exist
kubectl get crds | grep forgemaster.io

# Describe a CRD
kubectl describe crd agenttasks.forgemaster.io

# Check API resources
kubectl api-resources | grep forgemaster
```

### Test Creating Resources

```bash
# Create a test AgentTask
cat <<EOF | kubectl apply -f -
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
metadata:
  name: test-task
  namespace: default
spec:
  description: "Test task for CRD validation"
EOF

# Verify creation
kubectl get agenttasks
kubectl describe agenttask test-task

# Clean up
kubectl delete agenttask test-task
```

---

## Related

- [ADR-0005: Kubernetes-Native Architecture](../adr/0005-kubernetes-native-architecture.md)
- [05-k8s-deployment.md](05-k8s-deployment.md) — Full deployment architecture
- [Development Plan](../development-plan.md) — Implementation phases
