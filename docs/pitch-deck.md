# ForgeMaster Pitch Deck

Slide content for the 5-minute demo video. Use during non-terminal sections.

---

## Slide 1 — Title (0:00-0:10)

**ForgeMaster**

Autonomous Agent Orchestration for Kubernetes

*AgentForge Hackathon 2026 — Dmytro Shcherbatiuk*

---

## Slide 2 — The Problem (0:30-0:50)

**AI coding today is one-shot**

```
User → LLM → Code → Hope it works → Manual fix → Repeat
```

- No feedback loop
- No automated testing
- No deployment verification
- Human in the loop for every retry

---

## Slide 3 — The Solution (0:50-1:15)

**ForgeMaster: Control Theory + AI Agents**

```mermaid
flowchart LR
    A["Task"] --> B["Agent Pipeline"]
    B --> C["Deploy"]
    C --> D["Run Tests"]
    D --> E{"TCP Controller\nerror = failed / total"}
    E -->|"error > 0.7"| F["Swap Agent"]
    E -->|"0.2 - 0.7"| G["Adjust"]
    E -->|"< 0.2"| H["Continue"]
    F --> B
    G --> B
    H -->|"error ≈ 0"| I(["Done"])
```

| Error Signal | Action |
|---|---|
| > 0.7 | Swap agent |
| 0.5-0.7 | Add specialist |
| 0.2-0.5 | Adjust params |
| < 0.2 | Continue |

**Controller = math (microseconds, $0). Agents = Claude API (creative work, $$).**

---

## Slide 4 — Three Protocols (1:15-1:45)

**Standards-based, not reinvented**

| Protocol | Purpose | Standard |
|---|---|---|
| **MCP** | Agent-to-Tools | Anthropic |
| **A2A** | Agent-to-Agent | Google / Linux Foundation |
| **A2UI** | Agent-to-User | Google |

```mermaid
flowchart TB
    subgraph UI["A2UI Layer"]
        U["User"] <-->|"Declarative JSON"| P["React Portal"]
    end
    subgraph AGENTS["A2A Layer"]
        O["Orchestrator"] -->|"A2A"| AR["Architect"]
        AR -->|"A2A"| CG["Code Generator"]
        CG -->|"A2A"| DV["DevOps"]
        DV -->|"A2A"| TG["Test Generator"]
        TG -->|"A2A"| RV["Reviewer"]
        RV -->|"A2A"| O
    end
    subgraph TOOLS["MCP Layer"]
        FS["fm-mcp-filesystem"]
        DT["fm-mcp-devtools"]
        CT["fm-controller-agent-mcp"]
    end
    P <-->|"WebSocket"| O
    CG ---|"MCP"| FS
    CG ---|"MCP"| DT
    DV ---|"MCP"| DT
    O ---|"MCP"| CT
```

---

## Slide 5 — Agent Pipeline (1:45-2:00)

**Six agents, self-coordinating via A2A**

```mermaid
flowchart TD
    OR["Orchestrator\nWrite requirements\nCreate agents"] -->|"A2A"| AR["Architect\nAPI design, Gherkin tests\nArchitecture docs"]
    AR -->|"A2A"| CG["Code Generator\nSource code, Dockerfile\ndocker_build, Helm chart"]
    CG -->|"A2A"| DV["DevOps\nhelm_install\nVerify deployment"]
    DV -->|"A2A"| TG["Test Generator\nTest runner Docker + Helm Job"]
    TG -->|"A2A: deploy test Job"| DV
    DV -->|"A2A: test results"| TG
    TG -->|"Tests PASS"| RV["Reviewer\nCode quality, coverage, docs"]
    TG -->|"Tests FAIL (retry ≤ 3x)"| CG
    RV -->|"A2A: final report"| OR
```

Each agent:
- Own Kubernetes pod
- Own Claude API session
- Own MCP tools (filesystem, Docker, Helm)
- A2A endpoint for peer communication

---

## Slide 6 — Kubernetes-Native (2:00-2:15)

**Two Custom Resource Definitions**

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
spec:
  description: "Build a REST API..."
status:
  phase: Running
  error: 0.35
```

```yaml
apiVersion: forgemaster.io/v1alpha1
kind: Agent
spec:
  type: code-generator
  model:
    name: claude-sonnet-4-20250514
    maxTokens: 16384
  mcpServers:
    - name: fm-mcp-filesystem
    - name: fm-mcp-devtools
```

```mermaid
flowchart TB
    subgraph SYSTEM["forgemaster-system namespace"]
        ATC["AgentTask Controller"]
        AC["Agent Controller"]
        MCP1["fm-mcp-filesystem"]
        MCP2["fm-mcp-devtools"]
        MCP3["fm-controller-agent-mcp"]
    end
    subgraph TASK["task-f57bc331 namespace"]
        ORC["orchestrator"]
        ARCH["architect"]
        CGEN["code-generator"]
        DEVP["devops"]
        TGEN["test-generator"]
        REV["reviewer"]
        SVC["hello-world-service\nDeployment + Service"]
    end
    AC -->|"creates pods"| TASK
    ATC -->|"creates namespace"| TASK
    CGEN ---|"MCP"| MCP1
    CGEN ---|"MCP"| MCP2
    DEVP ---|"MCP"| MCP2
    ORC ---|"MCP"| MCP3
```

- Per-task namespace isolation
- Operator-managed lifecycle
- Resource quotas, RBAC, health checks

---

## Slides 7-10 — DEMO (2:15-3:50)

*No slides — show terminal and UI*

---

## Slide 11 — Engineering Quality (3:50-4:15)

**Production-grade, not a prototype**

| | |
|---|---|
| **Language** | Rust (5 crates, single responsibility each) |
| **Orchestration** | Kubernetes CRDs + Operators |
| **Deployment** | Helm charts + Ansible |
| **Environments** | Local (OrbStack) / Remote (ghcr.io) |
| **CI/CD** | GitHub Actions → ghcr.io |
| **Documentation** | 10 ADRs, architecture docs |
| **Observability** | Full audit trail per agent conversation |

---

## Slide 12 — Why ForgeMaster Should Win (4:15-4:50)

**1. Standards-based**
A2A + MCP + A2UI — composing open protocols, not reinventing them.
Any MCP server, any A2A agent can plug in.

**2. Kubernetes-native**
CRDs, operators, namespace isolation, Helm, RBAC.
Scales. Observable. Production-ready.

**3. Control theory meets AI**
TCP controller: microseconds, zero cost, deterministic.
LLM budget reserved for creative work only.

---

## Slide 13 — Close (4:50-5:00)

**ForgeMaster**

Autonomous agent orchestration,
powered by control theory,
built for Kubernetes.

*github.com/dshcherbatiuk/forgemaster*
