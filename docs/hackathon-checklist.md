# AgentForge Hackathon 2026 — Project Checklist

**Team:** CSM-101  
**Project:** Meta-Agent with TCP Controller  
**Submission Deadline:** February 9, 12:00 (4 days from start)

---

## Timeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HACKATHON TIMELINE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Feb 5 (Day 1)          Feb 6 (Day 2)         Feb 7 (Day 3)                │
│  ────────────           ────────────          ────────────                  │
│  □ Kick-off 17:00       □ Core services       □ Agents + A2A                │
│  □ Session 18:00        □ Session 17:00       □ Integration                 │
│  □ Setup Rust project   □ Mentor 18:00        □ Mentor support              │
│                                                                              │
│  Feb 8 (Day 4)          Feb 9 (Day 5)         Feb 10 (Day 6)               │
│  ────────────           ────────────          ────────────                  │
│  □ E2E testing          ⚠️ DEADLINE 12:00     □ Presentations 16:00         │
│  □ Mentor 13:00         □ Submit all          □ Top 3 announced 18:00       │
│  □ Video recording      □ Session 17:00       □ (Offline final TBD)         │
│  □ Pitch deck           □ Top 10 at 18:00                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Deliverables Checklist

### 1. Source Code (.zip) — 50% of evaluation

```
meta-agent/
├── □ Cargo.toml (workspace)
├── □ README.md (run instructions!)
├── crates/
│   ├── □ meta-agent-core/      # Shared types
│   ├── □ tcp-controller/       # TCP controller service
│   ├── □ agent-registry/       # Agent discovery
│   ├── □ agent-runtime/        # LLM agent execution
│   ├── □ a2a-rs/              # A2A protocol
│   └── □ k8s-operator/        # CRD controllers
├── helm/
│   └── □ meta-agent/          # Helm chart
├── ansible/
│   └── □ playbooks/           # Local deployment
└── docker/
    └── □ Dockerfile.*         # Container builds
```

**README.md Must Include:**
- [ ] Project description
- [ ] Architecture overview
- [ ] Prerequisites (Rust, Docker, Kind)
- [ ] Quick start (3-5 commands to run)
- [ ] Demo scenario
- [ ] API documentation

### 2. Video Demo (up to 5 minutes)

| Timestamp | Content | Duration |
|-----------|---------|----------|
| 0:00-0:30 | Problem statement | 30s |
| 0:30-1:30 | Architecture overview | 60s |
| 1:30-4:00 | Live demo | 150s |
| 4:00-5:00 | Next steps / AGI roadmap | 60s |

**Video Checklist:**
- [ ] Screen recording software ready
- [ ] Demo environment working
- [ ] Script written
- [ ] Practice run completed
- [ ] Recorded and edited
- [ ] Under 5 minutes

### 3. Pitch Deck (Presentation)

| Slide | Content |
|-------|---------|
| 1 | Title: Meta-Agent System with TCP Controller |
| 2 | Problem: Manual agent orchestration doesn't scale |
| 3 | Solution: Autonomous meta-agent with feedback loop |
| 4 | Architecture: TCP + Agents + A2A/MCP/A2UI |
| 5 | TCP Controller: Task-Context-Prediction orchestration |
| 6 | Demo: REST API generation example |
| 7 | Technical Stack: Rust, K8s, Claude |
| 8 | Differentiators: Control theory + AI agents |
| 9 | AGI Roadmap: Memory → Learning → Autonomy |
| 10 | Impact: 10x faster development, self-improving |

**Presentation Checklist:**
- [ ] Slides created
- [ ] Architecture diagrams included
- [ ] Demo screenshots/GIFs
- [ ] Practiced 5-min pitch

---

## Development Priorities

### Day 1 (Feb 5) — Foundation
- [x] Join Slack channel
- [ ] Attend kick-off 17:00
- [ ] Attend session 18:00
- [ ] Initialize Rust workspace
- [ ] Create basic project structure
- [ ] Set up CI (GitHub Actions)

### Day 2 (Feb 6) — Core Services
- [ ] Implement `tcp-controller` service
  - [ ] TCP computation (Task-Context-Prediction)
  - [ ] REST API (Axum)
  - [ ] Redis integration
- [ ] Implement `agent-registry` service
  - [ ] Register/deregister
  - [ ] Search by skill
  - [ ] Health checks
- [ ] Attend session 17:00
- [ ] Attend mentor session 18:00

### Day 3 (Feb 7) — Agents
- [ ] Implement `agent-runtime`
  - [ ] Claude API integration
  - [ ] System prompts
  - [ ] Streaming responses
- [ ] Implement A2A protocol basics
  - [ ] Agent Card
  - [ ] Task send/receive
- [ ] Test agent → controller → agent flow

### Day 4 (Feb 8) — Integration & Demo
- [ ] End-to-end test scenario
- [ ] Fix bugs from integration
- [ ] Attend mentor session 13:00
- [ ] Start video recording
- [ ] Create pitch deck

### Day 5 (Feb 9) — SUBMISSION
- [ ] Final testing
- [ ] Record video demo (if not done)
- [ ] Finalize pitch deck
- [ ] Write README
- [ ] Package .zip
- [ ] **SUBMIT BY 12:00** ⚠️

---

## MVP Scope (Must Have)

**Core Features:**
- [ ] TCP Controller with Task-Context-Prediction logic
- [ ] Agent Registry (register, search)
- [ ] 2 LLM Agents (Test Generator, Code Generator)
- [ ] Basic feedback loop
- [ ] One working demo scenario

**Demo Scenario:**
```
Input:  "Create a REST API endpoint for user registration"
        
System: 
1. TCP Controller receives task
2. Test Generator creates Gherkin tests
3. Code Generator writes Rust code
4. Tests run → feedback → iterate
5. Output: Working code + passing tests
```

---

## Nice to Have (If Time Permits)

- [ ] A2UI for rich user interface
- [ ] Kubernetes CRDs + Operator
- [ ] Helm chart packaging
- [ ] GitHub artifact storage
- [ ] Multiple agent types
- [ ] Memory system (Layer 1 AGI)

---

## Evaluation Criteria Mapping

| Criteria | Weight | How We Score High |
|----------|--------|-------------------|
| **Architecture & Design** | 20% | ✅ Comprehensive doc, diagrams, CRDs |
| **Technical Maturity** | 20% | Working demo, clean Rust code |
| **Scalability & Feasibility** | 10% | K8s-native, horizontal scaling |
| **Creativity & Originality** | 20% | ✅ TCP control theory is unique |
| **Impact & Usefulness** | 20% | Real automation, AGI roadmap |
| **Presentation & Demo** | 10% | Clear video, polished slides |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| LLM API rate limits | Cache responses, mock for testing |
| Integration complexity | Start E2E test early (Day 3) |
| Time pressure | MVP first, polish later |
| Demo fails live | Pre-record backup video |
| Rust compile times | Use `cargo watch`, incremental builds |

---

## Resources

- **Architecture Doc:** `meta-agent-architecture.md`
- **Anthropic API:** Provided by hackathon
- **Mentor Sessions:** Feb 6 18:00, Feb 8 13:00
- **Slack:** Join via email link

---

## Quick Commands

```bash
# Setup
cargo new meta-agent --name meta-agent
cd meta-agent

# Run TCP Controller
cargo run --package tcp-controller

# Run Agent Registry
cargo run --package agent-registry

# Run tests
cargo test --workspace

# Build Docker images
docker build -f docker/Dockerfile.tcp-controller -t tcp-controller .

# Deploy to Kind
kind create cluster --name meta-agent
helm install meta-agent ./helm/meta-agent
```

---

## Success Criteria

By Feb 9, 12:00 we need:

1. ✅ **Working demo** — one complete task flow
2. ✅ **5-min video** — problem → architecture → demo → future
3. ✅ **Pitch deck** — 10 slides
4. ✅ **Source code** — .zip with README
5. ✅ **Unique angle** — TCP Controller + AGI roadmap

**Goal:** Top 10 semi-finalist → Top 3 finalist → Win in Lviv! 🏆
