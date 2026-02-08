# ForgeMaster

Meta-agent system that discovers, creates, orchestrates, and manages task-specific AI agent subsystems in Kubernetes. Built for AgentForge Hackathon 2026.

## Prerequisites

- [Rust](https://rustup.rs/) (1.93+)
- [OrbStack](https://orbstack.dev/) (Kubernetes cluster + Docker runtime)
- [Docker](https://docs.docker.com/get-docker/) (for building agent service images)
- [Helm](https://helm.sh/) (3.x)
- [Ansible](https://docs.ansible.com/) (for orchestrating deployments)
- [Node.js](https://nodejs.org/) (22+, for UI)
- Anthropic API key

## Quick Start

### 1. Configure API key

Create `.env.forgemaster` in the project root:

```env
LLM_PROVIDER_API_KEY=sk-ant-your-key-here
```

See [LLM Provider Setup](docs/setup-llm-provider.md) for details.

### 2. Build

```bash
make all
```

### 3. Deploy to local cluster

```bash
make cluster
```

This starts OrbStack, builds Docker images, and deploys all services via Ansible + Helm. The UI URL is printed at the end of the deploy.

## Commands

| Command | Description |
|---------|-------------|
| `make all` | Format, lint, test, build |
| `make build` | `cargo build --release` |
| `make test` | `cargo test` |
| `make lint` | `cargo clippy` |
| `make fmt` | `cargo fmt` |
| `make cluster` | Deploy to local K8s cluster |
| `make cluster ENV=staging` | Deploy with a specific environment |
| `make cluster-clean` | Remove cluster deployments |
| `make cluster-reset` | Reset OrbStack Kubernetes |
| `make ui` | Format, lint, build UI |

## Project Structure

```
forgemaster/
├── crates/
│   ├── fm-controller-agenttask/  # AgentTask CRD + Controller
│   ├── fm-controller-agent/      # Agent CRD + Controller
│   └── fm-agent-runtime/         # Agent execution runtime
├── ui/                           # React + CopilotKit A2UI portal
├── ansible/
│   ├── site.yml
│   ├── environments/default/     # Local OrbStack environment
│   └── roles/                    # Per-service Ansible roles
└── docs/                         # Architecture & setup docs
```

## Documentation

- [Development Plan](docs/development-plan.md)
- [LLM Provider Setup](docs/setup-llm-provider.md)
- [Architecture Overview](docs/arch/01-overview.md)
- [Components](docs/arch/02-components.md)
- [Task Lifecycle](docs/arch/04-task-lifecycle.md)
- [Orchestrator Spec](docs/arch/04b-orchestrator-spec.md)
- [Requirements Preparation](docs/arch/04c-requirements-preparation.md)
- [CRD Specifications](docs/arch/09-crd-specifications.md)
