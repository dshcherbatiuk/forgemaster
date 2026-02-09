# ForgeMaster Demo Video Script (5 min)

## 0:00-0:10 — Introduction

**Screen:** ForgeMaster logo / title slide with tagline.

**Say:**
"Hi, I'm Dmytro. This is **ForgeMaster** — a meta-agent system that autonomously discovers, creates, orchestrates, and manages task-specific AI agent pipelines in Kubernetes. Built for the AgentForge Hackathon 2026."

---

## 0:10-0:30 — Hook

**Screen:** Terminal showing `kubectl get agents -A` with 6 agents running across a task namespace.

**Say:**
"What if you could describe a service in one sentence, and an autonomous system would design it, code it, build Docker images, deploy it to Kubernetes, write E2E tests, run them against the live service, and fix failures — all without human intervention? That's ForgeMaster."

---

## 0:30-1:15 — The Problem & Core Insight

**Screen:** Slide or diagram showing: "Single LLM agent" vs "ForgeMaster pipeline".

**Say:**
"Current AI coding tools are one-shot — generate code, hope it works. If it fails, you manually fix and retry. ForgeMaster borrows from control theory. We use a TCP feedback loop — Task, Context, Prediction — inspired by PID controllers. The key insight: orchestration decisions don't need intelligence. They need math. Is the error rate high? Swap the agent. Medium? Add a specialist. Low? Keep going. This runs in microseconds, costs zero, and is fully deterministic. We reserve the expensive LLM budget for what actually needs creativity — writing code, designing APIs, reviewing architecture."

---

## 1:15-2:15 — Architecture & Protocols

**Screen:** Architecture diagram (Mermaid or whiteboard). Show three protocol layers.

**Say:**
"ForgeMaster is built on three open protocols:"

"First — **MCP**, Anthropic's Model Context Protocol. Agents access tools — filesystem, Docker, Helm — through MCP servers running as Kubernetes pods. This is A2MCP: agents talk to tools."

"Second — **A2A**, Google's Agent-to-Agent protocol. Each agent has an Agent Card describing its skills. Agents discover each other, send task messages, and stream progress via SSE. This is how our six-agent pipeline self-coordinates — no central hub."

"Third — **A2UI**, Agent-to-User Interface. Agents generate declarative JSON that the React portal renders. Clarifying questions, live progress, test results, agent status — all without executing arbitrary code in the browser."

**Screen:** Show the K8s CRD YAML briefly.

"Everything is Kubernetes-native. Two custom CRDs — AgentTask and Agent. Each task gets its own namespace with isolated agents, MCP servers, and resource quotas. The operator pattern handles lifecycle, health checks, scalability, reliability, and cleanup automatically."

---

## 2:15-3:30 — Demo (pre-built task)

**Note:** Use the already completed `task-f57bc331` to avoid waiting and API costs. Show results, then walk through what happened.

**Screen:** Terminal — show the running pods and service.

```bash
kubectl get pods -n task-f57bc331
kubectl get svc -n task-f57bc331
```

**Say:**
"Here's a task I ran earlier — 'Build a REST API hello-world service with health check endpoint in Go.' The system created six agents in an isolated namespace. Let me walk you through what happened."

**Screen:** Show the workspace directory structure.

```bash
ls target/tasks/task-f57bc331/
```

**Say:**
"The **Orchestrator** analyzed the requirements and wrote the contract. It sent an A2A message to the **Architect**, who designed the API, wrote Gherkin acceptance tests, and created architecture documentation."

**Screen:** Show generated files — `docs/`, `features/`, source code.

"The Architect chained to the **Code Generator** via A2A — notice, this is agent-to-agent messaging, not the orchestrator directing. The Code Generator implemented the service, built a Docker image, and created Helm charts."

**Screen:** Show `helm/` directory and Dockerfile.

"Then the **DevOps agent** deployed to Kubernetes. The **Test Generator** built a test runner as a Kubernetes Job and asked DevOps to deploy it. If tests fail, it loops back to the Code Generator — up to three retries."

**Screen:** Call the live service to prove it works.

```bash
kubectl port-forward -n task-f57bc331 svc/hello-world-service-f57bc331 8080:80 &
curl http://localhost:8080/hello
curl http://localhost:8080/health
```

**Say:**
"And here's the actual service running in the cluster — built, deployed, and tested entirely by agents."

**Screen:** Show an audit log briefly.

```bash
ls target/tasks/task-f57bc331/audit/
```

"Every agent conversation is fully audited — every Claude API call, every tool invocation, every A2A message. Complete observability."

---

## 3:30-3:50 — Live Task Kickoff

**Note:** Start a new task live to show the process starting. Don't wait for completion — just show the first 20-30 seconds.

**Screen:** ForgeMaster UI portal.

**Action:** Submit a new task in the UI.

**Say:**
"Now let me start a fresh task so you can see the process in action."

**Screen:** Split terminal — `kubectl get pods -n task-xxx -w` watching pods appear.

"Watch the namespace — the orchestrator agent spins up first, analyzes the task, then creates the specialized agents one by one."

**Say (as pods appear):**
"Architect... Code Generator... DevOps... Test Generator... Reviewer — all provisioned automatically. Each one is a Kubernetes pod with its own Claude API session, MCP tools, and A2A endpoint. From here, the pipeline runs autonomously — exactly like the completed task we just saw."

---

## 3:50-4:15 — Engineering Quality & ADRs

**Screen:** Show `docs/adr/` directory listing, open one ADR.

**Say:**
"This isn't a weekend prototype. We have 10 Architecture Decision Records documenting every major choice — why Rust over Python, why custom Axum mock server over wiremock, why A2A over custom RPC."

**Screen:** Show the crate structure briefly.

"Five Rust crates, each with a single responsibility. Helm charts per service. Ansible for deployment orchestration. Two environments — local OrbStack for development, softserve pulling pre-built images from GitHub Container Registry."

"All backed by full audit trails — every agent decision is traceable."

---

## 4:15-4:50 — Why This Should Win

**Screen:** Summary slide with key differentiators.

**Say:**
"Three reasons ForgeMaster stands out:"

"**One** — Standards-based. A2A, MCP, A2UI — we're not reinventing protocols. We're composing open standards into a production system. Any MCP server, any A2A-compatible agent can plug in."

"**Two** — Kubernetes-native. This isn't a script running on a laptop. It's CRDs, operators, namespace isolation, Helm charts, RBAC. It scales. It's observable. It's how you'd actually run this in production."

"**Three** — Control theory meets AI. The TCP controller makes orchestration decisions in microseconds at zero cost. No LLM tokens wasted on 'should I retry?' Math handles that."

---

## 4:50-5:00 — Close

**Screen:** Terminal showing successful task completion — all tests passed.

**Say:**
"ForgeMaster — autonomous agent orchestration, powered by control theory, built for Kubernetes. Thank you."
