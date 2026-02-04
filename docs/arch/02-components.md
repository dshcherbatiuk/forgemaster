## Architecture Diagram

```mermaid
flowchart TB
    subgraph Input
        TI[Task Input]
    end

    subgraph Controller["TCP Controller"]
        T["T (Task)<br/>Current Analysis"]
        C["C (Context)<br/>History & Patterns"]
        P["P (Prediction)<br/>Anticipate & Adapt"]
        CS[Control Signal]
        T & C & P --> CS
    end

    subgraph Agents["Agent Pool"]
        TG["Test Generator Agent<br/>─────────────<br/>Analyzes task<br/>Generates Gherkin<br/>tests (setpoint)"]
        OR["Orchestrator Agent<br/>─────────────<br/>Agent Selection<br/>MCP Provisioning<br/>K8s CRD Management"]
        EX["Executor Agents<br/>─────────────<br/>Code Generator<br/>Reviewer<br/>Specialist"]
        TR["Test Runner Agent<br/>─────────────<br/>Runs Gherkin tests<br/>against output"]
        OV["Outcome Validator<br/>─────────────<br/>Verifies real-world<br/>results (API calls,<br/>emails, data checks)"]
        FB["Feedback Agent<br/>─────────────<br/>Collects metrics<br/>Calculates error<br/>Analyzes patterns"]
    end

    TI --> Controller
    CS --> TG & OR & EX
    TG --> TR
    EX --> TR
    TR --> OV
    OV --> FB
    FB -->|"Feedback Loop"| Controller
```

---

## Components

### 1. TCP Controller (Task-Context-Prediction)

The brain of the system. Receives task input and feedback, computes control signal.

**Responsibilities:**
- **T (Task):** Parse and understand incoming task, react to current error
- **C (Context):** Store and leverage history, learned patterns
- **P (Prediction):** Anticipate failures, adapt proactively
- Compute control signal to determine next action
- Decide: retry, spawn new agent, change strategy, or complete

### 2. Test Generator Agent

Creates the definition of success for each task using **Gherkin** syntax (Given-When-Then).

**Input:** Task description
**Output:** Set of E2E tests in Gherkin format

```mermaid
sequenceDiagram
    participant C as Controller
    participant TG as Test Generator Agent
    participant TR as Test Runner
    participant EX as Executor Agents

    C->>TG: Task: "Build REST API for user management"
    TG->>TG: Analyze task requirements
    TG->>TR: Generated Gherkin E2E Tests
    Note over TR: Feature: User Management API<br/>Scenario: Create user<br/>Scenario: Get user<br/>Scenario: Update user<br/>...
    C->>EX: Execute task
    EX->>TR: Output (API code)
    TR->>TR: Run tests against output
    TR->>C: Results: 3/5 passed (error=0.4)
```

**Example Gherkin Output:**

```gherkin
Feature: User Management API
  As a client
  I want to manage users via REST API
  So that I can perform CRUD operations

  Scenario: Create a new user
    Given the API is running
    When I send a POST request to "/users" with body:
      """
      {
        "name": "John Doe",
        "email": "john@example.com"
      }
      """
    Then the response status should be 201
    And the response should contain "id"

  Scenario: Get existing user
    Given a user exists with id "123"
    When I send a GET request to "/users/123"
    Then the response status should be 200
    And the response should contain "name"

  Scenario: Update user details
    Given a user exists with id "123"
    When I send a PUT request to "/users/123" with body:
      """
      {
        "name": "Jane Doe"
      }
      """
    Then the response status should be 200
    And the user name should be "Jane Doe"

  Scenario: Delete user
    Given a user exists with id "123"
    When I send a DELETE request to "/users/123"
    Then the response status should be 204
    And the user should no longer exist

  Scenario: Invalid input returns error
    Given the API is running
    When I send a POST request to "/users" with body:
      """
      {
        "invalid": "data"
      }
      """
    Then the response status should be 400
    And the response should contain "error"
```

### 3. Orchestrator Agent

An agent that provisions and manages execution resources.

**Responsibilities:**
- Select appropriate agents for the task
- Create new agents if needed (dynamically define new Agent CRDs)
- Provision MCP servers
- Manage K8s CRDs (AgentTask, Agent, MCPServer, TestSuite)
- Manage lifecycle (start, stop, scale)

### 4. Executor Agents

Task-specific agents that do the actual work.

**Types:**
- Code Generator
- Code Reviewer
- Data Processor
- Research Agent
- Specialist Agents (created on demand)

### 5. Test Runner Agent

An agent that executes Gherkin E2E tests against the output using BDD frameworks.

**Input:** Gherkin feature files + Agent output
**Output:** Test results (pass/fail per scenario)

**Supported Runners:**
- Python: `behave`, `pytest-bdd`
- JavaScript: `cucumber-js`
- Java: `cucumber-jvm`
- Go: `godog`

### 6. Outcome Validator

Verifies real-world results beyond self-generated tests. Since tests are created by the system itself, they could have false positives. Outcome validation checks that the task **actually worked**.

**Why needed:** Tests verify the HOW (implementation). Outcomes verify the WHAT (it actually worked).

**Validation by Task Type:**

| Task Type | Outcome Verification |
|-----------|---------------------|
| Ticket Booking | Confirmation email received, booking ID valid in external system |
| ETL Pipeline | Data exists in target DB, row counts match, checksums valid |
| API Development | External client can call endpoints, integration tests pass |
| ML Training | Metrics improve on held-out validation set |
| Test Automation | Generated tests actually catch bugs when code is broken |

**Input:** Task output + task type
**Output:** Outcome validation results (verified/failed per check)

### 7. Feedback Collector Agent

An agent that gathers metrics, analyzes results, and calculates the error signal for the TCP Controller.

**Responsibilities:**
- Collect test results from Test Runner Agent
- Collect outcome validation results from Outcome Validator
- Gather execution metrics (time, tokens, retries)
- Calculate combined error signal
- Analyze failure patterns
- Provide recommendations for next iteration

**Error Signal Calculation:**
```
error = (failed_tests + failed_outcomes) / total_checks
```

**Metrics:**
- Test pass rate
- Outcome validation rate
- Execution time
- Token usage
- Retry count
- Agent performance trends

---
