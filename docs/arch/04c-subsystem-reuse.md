# Subsystem Reuse

## Overview

**Subsystem reuse** is a core capability of ForgeMaster. After successfully completing tasks, the system extracts reusable agent combinations as templates for future similar tasks.

## Concept

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        SUBSYSTEM REUSE CYCLE                             │
│                                                                          │
│   Task 1: "Build REST API for products"                                  │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Agents: [test-gen, code-gen, reviewer]                         │   │
│   │  MCPs: [github, filesystem]                                      │   │
│   │  Result: SUCCESS                                                 │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                              │                                           │
│                              ▼                                           │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                 EXTRACT SUBSYSTEM TEMPLATE                       │   │
│   │                                                                  │   │
│   │  Name: "rest-api-rust-axum"                                      │   │
│   │  Agents: [test-gen, code-gen, reviewer]                         │   │
│   │  MCPs: [github, filesystem]                                      │   │
│   │  Success rate: 100% (1/1)                                        │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                              │                                           │
│                              ▼                                           │
│   Task 2: "Build REST API for orders"                                    │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │  Similarity detected: 95% match with "rest-api-rust-axum"       │   │
│   │  → Reuse subsystem template                                      │   │
│   │  → Faster startup, proven combination                            │   │
│   └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Subsystem Definition

### Subsystem Template Schema

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: SubsystemTemplate
metadata:
  name: rest-api-rust-axum
  namespace: metaagent-system
  labels:
    forgemaster.io/task-type: web-api
    forgemaster.io/language: rust
    forgemaster.io/framework: axum
spec:
  # Description
  description: |
    Proven agent combination for building REST APIs in Rust with Axum framework.
    Includes test generation, code generation, and code review.

  # Task matching criteria
  matching:
    task_types:
      - web-api
      - rest-api
    languages:
      - rust
    frameworks:
      - axum
    keywords:
      - api
      - rest
      - crud
      - endpoints

  # Agent configurations
  agents:
    - type: test-generator
      config:
        model: claude-3-sonnet
        prompt_template: rust-api-tests
        skills: [gherkin-generation, rust-testing]

    - type: code-generator
      config:
        model: claude-3-sonnet
        prompt_template: rust-axum-api
        skills: [rust-code-generation, axum-framework]

    - type: reviewer
      config:
        model: claude-3-opus
        prompt_template: rust-security-review
        skills: [code-review, security-review]

  # MCP server configurations
  mcp_servers:
    - type: github-mcp
      required: true
    - type: filesystem-mcp
      required: true
    - type: postgres-mcp
      required: false
      condition: "database in features"

  # Execution flow
  execution_order:
    - stage: 1
      agents: [test-generator]
      parallel: false
    - stage: 2
      agents: [code-generator]
      parallel: false
    - stage: 3
      agents: [reviewer]
      parallel: false

  # Performance metrics
  metrics:
    times_used: 15
    success_rate: 0.87
    avg_iterations: 4.2
    avg_duration_minutes: 25

status:
  last_used: "2026-02-03T14:30:00Z"
  created_from_task: "task-a1b2c3d4"
  version: 3
```

## Subsystem Registry

### Storage

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      SUBSYSTEM REGISTRY                                  │
│                                                                          │
│   Storage: Redis + Persistent Volume                                     │
│                                                                          │
│   Keys:                                                                  │
│   ─────                                                                  │
│   subsystem:index:task_type:web-api → [rest-api-rust-axum, ...]         │
│   subsystem:index:language:rust → [rest-api-rust-axum, ...]             │
│   subsystem:template:rest-api-rust-axum → {template YAML}               │
│   subsystem:metrics:rest-api-rust-axum → {usage stats}                  │
│   subsystem:history:rest-api-rust-axum → [task-ids...]                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/subsystems` | List all subsystem templates |
| GET | `/api/v1/subsystems/{name}` | Get specific template |
| POST | `/api/v1/subsystems/match` | Find matching templates for task |
| POST | `/api/v1/subsystems` | Create new template |
| PUT | `/api/v1/subsystems/{name}` | Update template |
| DELETE | `/api/v1/subsystems/{name}` | Delete template |

## Template Extraction

### When to Extract

```yaml
extraction_triggers:
  # Primary trigger: successful task completion
  - event: task_completed
    condition: status == SUCCESS
    action: analyze_for_extraction

  # Additional triggers
  - event: multiple_similar_successes
    condition: same_agent_combo_success >= 3
    action: create_template
```

### Extraction Algorithm

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TEMPLATE EXTRACTION ALGORITHM                         │
│                                                                          │
│  Input: Completed task with SUCCESS status                               │
│                                                                          │
│  Step 1: Analyze task characteristics                                    │
│    - task_type: web-api                                                  │
│    - language: rust                                                      │
│    - framework: axum                                                     │
│    - features: [auth, validation]                                        │
│                                                                          │
│  Step 2: Extract agent configuration                                     │
│    - List all agents used                                                │
│    - Capture final configurations (post any adjustments)                 │
│    - Record prompt templates                                             │
│                                                                          │
│  Step 3: Extract MCP configuration                                       │
│    - List MCP servers used                                               │
│    - Identify required vs optional                                       │
│                                                                          │
│  Step 4: Check for existing similar template                             │
│    - Query registry by task characteristics                              │
│    - If match found with >80% similarity:                                │
│        → Update existing template (increment version)                    │
│    - If no match:                                                        │
│        → Create new template                                             │
│                                                                          │
│  Step 5: Calculate matching criteria                                     │
│    - Extract keywords from task description                              │
│    - Identify task type patterns                                         │
│    - Set matching thresholds                                             │
│                                                                          │
│  Step 6: Store template                                                  │
│    - Save to Subsystem Registry                                          │
│    - Index by task_type, language, framework                             │
│    - Link to source task for reference                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Template Matching

### Matching Algorithm

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TEMPLATE MATCHING ALGORITHM                           │
│                                                                          │
│  Input: New task requirements                                            │
│                                                                          │
│  Step 1: Extract task features                                           │
│    features = {                                                          │
│      task_type: "web-api",                                               │
│      language: "rust",                                                   │
│      framework: "axum",                                                  │
│      keywords: ["api", "users", "crud"]                                  │
│    }                                                                     │
│                                                                          │
│  Step 2: Query candidate templates                                       │
│    candidates = registry.find_by_indices(                                │
│      task_type: features.task_type,                                      │
│      language: features.language                                         │
│    )                                                                     │
│                                                                          │
│  Step 3: Score each candidate                                            │
│    for template in candidates:                                           │
│      score = calculate_similarity(features, template.matching)           │
│      # Weighted scoring:                                                 │
│      #   task_type match: 0.3                                            │
│      #   language match: 0.25                                            │
│      #   framework match: 0.25                                           │
│      #   keyword overlap: 0.2                                            │
│                                                                          │
│  Step 4: Rank by score + success rate                                    │
│    final_score = score * 0.7 + template.success_rate * 0.3              │
│                                                                          │
│  Step 5: Return best match if score > threshold                          │
│    if best_score > 0.75:                                                 │
│      return best_template                                                │
│    else:                                                                 │
│      return None  # No suitable template, create fresh                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Match Response

```json
{
  "task_id": "task-xyz789",
  "match_found": true,
  "template": {
    "name": "rest-api-rust-axum",
    "score": 0.92,
    "success_rate": 0.87,
    "times_used": 15
  },
  "recommendation": "USE_TEMPLATE",
  "customizations_needed": [
    "Add authentication-related MCP if auth feature requested"
  ]
}
```

## Template Application

### Instantiation Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TEMPLATE INSTANTIATION                                │
│                                                                          │
│  Input: Task + Matched Template                                          │
│                                                                          │
│  Step 1: Create task namespace                                           │
│    kubectl create namespace task-xyz789                                  │
│                                                                          │
│  Step 2: Customize template for task                                     │
│    - Replace placeholders with task-specific values                      │
│    - Adjust prompts with task requirements                               │
│    - Add/remove optional MCPs based on features                          │
│                                                                          │
│  Step 3: Deploy agents from template                                     │
│    for agent_config in template.agents:                                  │
│      customized = customize(agent_config, task)                          │
│      deploy_agent(customized, namespace)                                 │
│                                                                          │
│  Step 4: Deploy MCP servers                                              │
│    for mcp_config in template.mcp_servers:                               │
│      if mcp_config.required or condition_met(mcp_config):                │
│        deploy_mcp(mcp_config, namespace)                                 │
│                                                                          │
│  Step 5: Configure execution flow                                        │
│    - Set up agent communication per template.execution_order             │
│    - Initialize context store                                            │
│                                                                          │
│  Step 6: Start execution                                                 │
│    - Hand off to Orchestrator                                            │
│    - Orchestrator follows template execution order                       │
└─────────────────────────────────────────────────────────────────────────┘
```

## Template Evolution

### Versioning

```yaml
# Template versioning strategy
versioning:
  # Increment patch for minor updates
  patch:
    - Prompt tweaks
    - Model temperature adjustments

  # Increment minor for additions
  minor:
    - New optional agent added
    - New MCP server option

  # Increment major for breaking changes
  major:
    - Agent removed
    - Execution order changed
    - Required component changed

# Example versions
# v1.0.0 - Initial template
# v1.1.0 - Added optional security-reviewer agent
# v1.1.1 - Tweaked code-generator prompt
# v2.0.0 - Changed from sonnet to opus for code-generator
```

### Learning from Usage

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TEMPLATE LEARNING                                     │
│                                                                          │
│  After each task using template:                                         │
│                                                                          │
│  1. Record outcome                                                       │
│     - Success/Failure                                                    │
│     - Number of iterations                                               │
│     - Agent swaps/additions made                                         │
│                                                                          │
│  2. Analyze patterns                                                     │
│     - If same adjustment made 3+ times:                                  │
│         → Incorporate into template                                      │
│     - If failure rate > 30%:                                             │
│         → Flag for review                                                │
│                                                                          │
│  3. Update template                                                      │
│     - Merge successful adjustments                                       │
│     - Update metrics                                                     │
│     - Bump version if changed                                            │
│                                                                          │
│  Example:                                                                │
│    Template "rest-api-rust-axum" used for 10 tasks                       │
│    7 tasks added "input-validator" agent during execution                │
│    → Update template to include "input-validator" as default             │
└─────────────────────────────────────────────────────────────────────────┘
```

## Subsystem Composition

### Composing Multiple Subsystems

```yaml
# Complex task might use multiple subsystems
task: "Build full-stack e-commerce platform"

composed_subsystems:
  - name: rest-api-rust-axum
    scope: backend-api
    customization:
      features: [auth, payments, inventory]

  - name: react-frontend
    scope: web-frontend
    customization:
      features: [cart, checkout, user-dashboard]

  - name: postgres-schema
    scope: database
    customization:
      tables: [users, products, orders]

coordination:
  # How subsystems interact
  - backend-api depends_on database
  - web-frontend depends_on backend-api
```

### Subsystem Communication

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Database      │     │   Backend API   │     │    Frontend     │
│   Subsystem     │◄────│   Subsystem     │◄────│   Subsystem     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                       │                       │
        └───────────────────────┴───────────────────────┘
                                │
                    ┌───────────────────────┐
                    │   Shared Context      │
                    │   (Cross-subsystem)   │
                    └───────────────────────┘
```

## Best Practices

### Template Creation

1. **Wait for proven success** - Extract after 2-3 similar successful tasks
2. **Keep templates focused** - One template per task type/language combo
3. **Document customization points** - Mark what varies between uses
4. **Include failure handling** - Capture what adjustments were needed

### Template Usage

1. **Don't force-fit** - If match score < 0.75, create fresh
2. **Allow customization** - Templates are starting points, not rigid
3. **Track modifications** - If template consistently needs changes, update it
4. **Monitor success rates** - Retire templates with low success

### Template Maintenance

1. **Regular review** - Quarterly review of template performance
2. **Prune unused** - Archive templates not used in 90 days
3. **Merge similar** - Combine templates with >90% overlap
4. **Version carefully** - Major versions for breaking changes only
