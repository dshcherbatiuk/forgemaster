## Backend Services: Rust

All backend services are written in **Rust** for performance, safety, and low resource footprint.

### Rust Workspace Structure

```
meta-agent/
├── Cargo.toml                 # Workspace root
├── Cargo.lock
├── rust-toolchain.toml
├── .cargo/
│   └── config.toml
│
├── crates/
│   ├── meta-agent-core/       # Shared types, traits, utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs
│   │       ├── error.rs
│   │       └── config.rs
│   │
│   ├── tcp-controller/        # TCP Controller service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── controller.rs
│   │       ├── feedback.rs
│   │       └── api.rs
│   │
│   ├── agent-registry/        # Agent Registry service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── registry.rs
│   │       ├── health.rs
│   │       └── api.rs
│   │
│   ├── agent-runtime/         # Agent execution runtime
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── agent.rs
│   │       ├── llm.rs
│   │       └── executor.rs
│   │
│   ├── a2a-rs/                # A2A Protocol implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── server.rs
│   │       ├── types.rs
│   │       └── transport.rs
│   │
│   ├── a2ui-rs/               # A2UI Protocol implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── components.rs
│   │       ├── renderer.rs
│   │       └── streaming.rs
│   │
│   ├── mcp-client/            # MCP Client implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       └── tools.rs
│   │
│   └── k8s-operator/          # Kubernetes Operator (kube-rs)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── crds.rs
│           ├── controllers/
│           │   ├── mod.rs
│           │   ├── agent_task.rs
│           │   ├── agent.rs
│           │   └── mcp_server.rs
│           └── reconcilers.rs
│
├── docker/
│   ├── Dockerfile.tcp-controller
│   ├── Dockerfile.agent-registry
│   ├── Dockerfile.agent-runtime
│   └── Dockerfile.k8s-operator
│
└── tests/
    ├── integration/
    └── e2e/
```

### Workspace Cargo.toml

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "crates/meta-agent-core",
    "crates/tcp-controller",
    "crates/agent-registry",
    "crates/agent-runtime",
    "crates/a2a-rs",
    "crates/a2ui-rs",
    "crates/mcp-client",
    "crates/k8s-operator",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "Apache-2.0"
repository = "https://github.com/csm-101/meta-agent"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Web framework
axum = { version = "0.7", features = ["ws", "macros"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP client
reqwest = { version = "0.11", features = ["json", "stream"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Kubernetes
kube = { version = "0.87", features = ["runtime", "derive"] }
k8s-openapi = { version = "0.20", features = ["v1_28"] }

# LLM
async-openai = "0.18"  # Works with Claude API

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Config
config = "0.14"
dotenvy = "0.15"

# Testing
tokio-test = "0.4"
wiremock = "0.5"
```

### Core Types (meta-agent-core)

```rust
// crates/meta-agent-core/src/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub proficiency: f32,  // 0.0 - 1.0
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub a2a: Option<A2ACapability>,
    pub a2ui: Option<A2UICapability>,
    pub mcp: Option<MCPCapability>,
    pub streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ACapability {
    pub enabled: bool,
    pub endpoint: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UICapability {
    pub enabled: bool,
    pub supported_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCapability {
    pub enabled: bool,
    pub tools: Vec<String>,
}

/// TCP Controller coefficients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TCPCoefficients {
    pub task: f32,       // T weight
    pub context: f32,    // C weight
    pub prediction: f32, // P weight
}

impl Default for TCPCoefficients {
    fn default() -> Self {
        Self {
            task: 1.0,
            context: 0.5,
            prediction: 0.3,
        }
    }
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
}

/// Agent health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Test result from Gherkin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub scenario: String,
    pub status: TestStatus,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

/// Error calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSignal {
    pub value: f32,           // 0.0 - 1.0
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub iteration: u32,
}

impl ErrorSignal {
    pub fn from_test_results(results: &[TestResult], iteration: u32) -> Self {
        let total = results.len() as u32;
        let passed = results.iter().filter(|r| r.status == TestStatus::Passed).count() as u32;
        let failed = total - passed;
        
        Self {
            value: if total > 0 { failed as f32 / total as f32 } else { 1.0 },
            tests_total: total,
            tests_passed: passed,
            tests_failed: failed,
            iteration,
        }
    }
}
```

### TCP Controller Service

```rust
// crates/tcp-controller/src/main.rs

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

mod controller;
mod feedback;
mod api;

use controller::TCPController;
use meta_agent_core::config::Config;

#[derive(Clone)]
pub struct AppState {
    controller: Arc<RwLock<TCPController>>,
    redis: redis::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("tcp_controller=debug,tower_http=debug")
        .json()
        .init();

    // Load config
    let config = Config::load()?;
    
    // Connect to Redis
    let redis = redis::Client::open(config.redis_url.clone())?;
    
    // Initialize TCP Controller
    let controller = TCPController::new(config.tcp_coefficients.clone());
    
    let state = AppState {
        controller: Arc::new(RwLock::new(controller)),
        redis,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/tasks", post(api::create_task))
        .route("/api/v1/tasks/:id", get(api::get_task))
        .route("/api/v1/tasks/:id/feedback", post(api::submit_feedback))
        .route("/api/v1/control-signal", post(api::compute_control_signal))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config.port);
    info!("TCP Controller listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
```

```rust
// crates/tcp-controller/src/controller.rs

use meta_agent_core::types::{TCPCoefficients, ErrorSignal};
use tracing::instrument;

/// TCP Controller implementing Task-Context-Prediction feedback loop
pub struct TCPController {
    coefficients: TCPCoefficients,
    context_history: Vec<ErrorSignal>,
    max_history: usize,
}

impl TCPController {
    pub fn new(coefficients: TCPCoefficients) -> Self {
        Self {
            coefficients,
            context_history: Vec::new(),
            max_history: 100,
        }
    }
    
    /// Compute control signal based on TCP model
    #[instrument(skip(self))]
    pub fn compute_control_signal(&mut self, current_error: &ErrorSignal) -> ControlSignal {
        // T (Task) - Proportional response to current error
        let t_action = self.coefficients.task * current_error.value;
        
        // C (Context) - Integral of historical errors
        let c_action = self.compute_context_action();
        
        // D (Prediction) - Derivative based on error trend
        let p_action = self.compute_prediction_action(current_error);
        
        // Store in history
        self.context_history.push(current_error.clone());
        if self.context_history.len() > self.max_history {
            self.context_history.remove(0);
        }
        
        // Combined control signal
        let total = t_action + c_action + p_action;
        
        ControlSignal {
            action: self.determine_action(total, current_error),
            t_component: t_action,
            c_component: c_action,
            p_component: p_action,
            total_signal: total,
        }
    }
    
    fn compute_context_action(&self) -> f32 {
        if self.context_history.is_empty() {
            return 0.0;
        }
        
        // Average error over history (integral approximation)
        let sum: f32 = self.context_history.iter().map(|e| e.value).sum();
        let avg = sum / self.context_history.len() as f32;
        
        self.coefficients.context * avg
    }
    
    fn compute_prediction_action(&self, current: &ErrorSignal) -> f32 {
        if self.context_history.is_empty() {
            return 0.0;
        }
        
        // Derivative: rate of change
        let previous = self.context_history.last().unwrap();
        let derivative = current.value - previous.value;
        
        self.coefficients.prediction * derivative
    }
    
    fn determine_action(&self, signal: f32, error: &ErrorSignal) -> ControlAction {
        match signal {
            s if s > 0.5 => ControlAction::MajorChange {
                reason: "High error - swap agent or change approach".into(),
                swap_agent: true,
            },
            s if s > 0.2 => ControlAction::ModerateChange {
                reason: "Medium error - add reviewer or adjust params".into(),
                add_agent: Some("reviewer".into()),
            },
            s if s > 0.0 => ControlAction::MinorChange {
                reason: "Low error - fine-tune and retry".into(),
            },
            _ => ControlAction::Complete {
                reason: "Error within threshold".into(),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlSignal {
    pub action: ControlAction,
    pub t_component: f32,
    pub c_component: f32,
    pub p_component: f32,
    pub total_signal: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlAction {
    MajorChange { reason: String, swap_agent: bool },
    ModerateChange { reason: String, add_agent: Option<String> },
    MinorChange { reason: String },
    Complete { reason: String },
}
```

### Agent Registry Service

```rust
// crates/agent-registry/src/main.rs

use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::{State, Path, Query},
    Json,
};
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod registry;
mod health;
mod api;

use registry::{AgentRegistry, AgentRegistration};

#[derive(Clone)]
pub struct AppState {
    registry: Arc<RwLock<AgentRegistry>>,
    redis: redis::aio::ConnectionManager,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("agent_registry=debug")
        .json()
        .init();

    let config = meta_agent_core::config::Config::load()?;
    
    // Connect to Redis
    let client = redis::Client::open(config.redis_url.clone())?;
    let redis = client.get_connection_manager().await?;
    
    // Initialize registry
    let registry = AgentRegistry::new(redis.clone());
    
    let state = AppState {
        registry: Arc::new(RwLock::new(registry)),
        redis,
    };
    
    // Start health checker background task
    let health_state = state.clone();
    tokio::spawn(async move {
        health::run_health_checker(health_state).await;
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        // Registration
        .route("/agents/register", post(api::register_agent))
        .route("/agents/:id", get(api::get_agent))
        .route("/agents/:id", delete(api::deregister_agent))
        .route("/agents/:id/heartbeat", put(api::heartbeat))
        // Discovery
        .route("/agents/search", get(api::search_agents))
        .route("/agents/skills", get(api::list_skills))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.registry_port);
    info!("Agent Registry listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

```rust
// crates/agent-registry/src/registry.rs

use meta_agent_core::types::{Skill, Capabilities, HealthStatus};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

const AGENTS_KEY: &str = "meta-agent:registry:agents";
const AGENT_PREFIX: &str = "meta-agent:registry:agent:";
const SKILLS_INDEX: &str = "meta-agent:registry:skills:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub skills: Vec<Skill>,
    pub capabilities: Capabilities,
    pub health: HealthStatus,
    pub current_load: u32,
    pub max_concurrent_tasks: u32,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub endpoint: String,
    pub skills: Vec<Skill>,
    pub capabilities: Capabilities,
    #[serde(default = "default_max_tasks")]
    pub max_concurrent_tasks: u32,
}

fn default_max_tasks() -> u32 { 5 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub skill: Option<String>,
    pub capability: Option<String>,
    pub healthy: Option<bool>,
    pub min_proficiency: Option<f32>,
    pub max_load: Option<u32>,
}

pub struct AgentRegistry {
    redis: redis::aio::ConnectionManager,
    lease_ttl_secs: u64,
}

impl AgentRegistry {
    pub fn new(redis: redis::aio::ConnectionManager) -> Self {
        Self {
            redis,
            lease_ttl_secs: 60,
        }
    }
    
    /// Register a new agent
    pub async fn register(&mut self, req: RegisterRequest) -> anyhow::Result<AgentRegistration> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let registration = AgentRegistration {
            id: id.clone(),
            name: req.name,
            endpoint: req.endpoint,
            skills: req.skills.clone(),
            capabilities: req.capabilities,
            health: HealthStatus::Healthy,
            current_load: 0,
            max_concurrent_tasks: req.max_concurrent_tasks,
            registered_at: now,
            last_heartbeat: now,
        };
        
        // Store agent data
        let key = format!("{}{}", AGENT_PREFIX, id);
        let data = serde_json::to_string(&registration)?;
        self.redis.set_ex(&key, &data, self.lease_ttl_secs).await?;
        
        // Add to agents set
        self.redis.sadd(AGENTS_KEY, &id).await?;
        
        // Index by skills
        for skill in &req.skills {
            let skill_key = format!("{}{}", SKILLS_INDEX, skill.name);
            self.redis.sadd(&skill_key, &id).await?;
        }
        
        Ok(registration)
    }
    
    /// Update heartbeat
    pub async fn heartbeat(
        &mut self, 
        id: &str, 
        current_load: Option<u32>
    ) -> anyhow::Result<()> {
        let key = format!("{}{}", AGENT_PREFIX, id);
        
        // Get current registration
        let data: Option<String> = self.redis.get(&key).await?;
        let mut reg: AgentRegistration = match data {
            Some(d) => serde_json::from_str(&d)?,
            None => anyhow::bail!("Agent not found: {}", id),
        };
        
        // Update
        reg.last_heartbeat = Utc::now();
        reg.health = HealthStatus::Healthy;
        if let Some(load) = current_load {
            reg.current_load = load;
        }
        
        // Save with TTL refresh
        let data = serde_json::to_string(&reg)?;
        self.redis.set_ex(&key, &data, self.lease_ttl_secs).await?;
        
        Ok(())
    }
    
    /// Search for agents
    pub async fn search(&mut self, query: SearchQuery) -> anyhow::Result<Vec<AgentRegistration>> {
        let mut agent_ids: Vec<String> = if let Some(skill) = &query.skill {
            // Get agents with this skill
            let skill_key = format!("{}{}", SKILLS_INDEX, skill);
            self.redis.smembers(&skill_key).await?
        } else {
            // Get all agents
            self.redis.smembers(AGENTS_KEY).await?
        };
        
        let mut results = Vec::new();
        
        for id in agent_ids {
            let key = format!("{}{}", AGENT_PREFIX, id);
            let data: Option<String> = self.redis.get(&key).await?;
            
            if let Some(d) = data {
                let reg: AgentRegistration = serde_json::from_str(&d)?;
                
                // Apply filters
                if let Some(true) = query.healthy {
                    if reg.health != HealthStatus::Healthy {
                        continue;
                    }
                }
                
                if let Some(max_load) = query.max_load {
                    if reg.current_load > max_load {
                        continue;
                    }
                }
                
                if let Some(min_prof) = query.min_proficiency {
                    if let Some(skill_name) = &query.skill {
                        let has_skill = reg.skills.iter()
                            .any(|s| &s.name == skill_name && s.proficiency >= min_prof);
                        if !has_skill {
                            continue;
                        }
                    }
                }
                
                if let Some(cap) = &query.capability {
                    let has_cap = match cap.as_str() {
                        "a2a" => reg.capabilities.a2a.as_ref().map(|c| c.enabled).unwrap_or(false),
                        "a2ui" => reg.capabilities.a2ui.as_ref().map(|c| c.enabled).unwrap_or(false),
                        "mcp" => reg.capabilities.mcp.as_ref().map(|c| c.enabled).unwrap_or(false),
                        _ => true,
                    };
                    if !has_cap {
                        continue;
                    }
                }
                
                results.push(reg);
            }
        }
        
        // Sort by load (ascending)
        results.sort_by_key(|r| r.current_load);
        
        Ok(results)
    }
    
    /// Deregister an agent
    pub async fn deregister(&mut self, id: &str) -> anyhow::Result<()> {
        let key = format!("{}{}", AGENT_PREFIX, id);
        
        // Get registration to clean up skill indexes
        let data: Option<String> = self.redis.get(&key).await?;
        if let Some(d) = data {
            let reg: AgentRegistration = serde_json::from_str(&d)?;
            
            // Remove from skill indexes
            for skill in &reg.skills {
                let skill_key = format!("{}{}", SKILLS_INDEX, skill.name);
                self.redis.srem(&skill_key, id).await?;
            }
        }
        
        // Remove from agents set
        self.redis.srem(AGENTS_KEY, id).await?;
        
        // Delete agent data
        self.redis.del(&key).await?;
        
        Ok(())
    }
    
    /// Get all available skills
    pub async fn list_skills(&mut self) -> anyhow::Result<Vec<SkillInfo>> {
        let agents: Vec<String> = self.redis.smembers(AGENTS_KEY).await?;
        let mut skill_map: HashMap<String, SkillInfo> = HashMap::new();
        
        for id in agents {
            let key = format!("{}{}", AGENT_PREFIX, id);
            let data: Option<String> = self.redis.get(&key).await?;
            
            if let Some(d) = data {
                let reg: AgentRegistration = serde_json::from_str(&d)?;
                
                for skill in reg.skills {
                    let entry = skill_map.entry(skill.name.clone()).or_insert(SkillInfo {
                        name: skill.name,
                        agent_count: 0,
                        total_proficiency: 0.0,
                    });
                    entry.agent_count += 1;
                    entry.total_proficiency += skill.proficiency;
                }
            }
        }
        
        Ok(skill_map.into_values()
            .map(|mut s| {
                s.total_proficiency /= s.agent_count as f32; // Convert to average
                s
            })
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub agent_count: u32,
    #[serde(rename = "avgProficiency")]
    pub total_proficiency: f32,
}
```

### A2A Protocol Implementation

```rust
// crates/a2a-rs/src/lib.rs

pub mod client;
pub mod server;
pub mod types;
pub mod transport;

pub use client::A2AClient;
pub use server::A2AServer;
pub use types::*;
```

```rust
// crates/a2a-rs/src/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent Card - describes agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub version: String,
    pub endpoint: String,
    pub capabilities: AgentCapabilities,
    pub skills: Vec<AgentSkill>,
    pub authentication: Option<AuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub push_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub name: String,
    pub description: String,
    pub input_modes: Vec<String>,
    pub output_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub authorization_url: Option<String>,
}

/// A2A Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub session_id: Option<String>,
    pub status: TaskStatus,
    pub message: Message,
    pub artifacts: Vec<Artifact>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MessagePart {
    Text { text: String },
    File { uri: String, mime_type: String },
    Data { data: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub content: ArtifactContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArtifactContent {
    Text(String),
    Binary(Vec<u8>),
    Json(serde_json::Value),
}

/// JSON-RPC request/response for A2A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

```rust
// crates/a2a-rs/src/client.rs

use crate::types::*;
use reqwest::Client;
use tokio_stream::Stream;
use futures::StreamExt;

pub struct A2AClient {
    http: Client,
    base_url: String,
}

impl A2AClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: endpoint.to_string(),
        }
    }
    
    /// Fetch agent card for discovery
    pub async fn get_agent_card(&self) -> anyhow::Result<AgentCard> {
        let url = format!("{}/.well-known/agent.json", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let card = resp.json::<AgentCard>().await?;
        Ok(card)
    }
    
    /// Send a task to the agent
    pub async fn send_task(&self, message: Message) -> anyhow::Result<Task> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "tasks/send".to_string(),
            params: serde_json::json!({
                "message": message
            }),
        };
        
        let resp = self.http
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?;
            
        let rpc_resp = resp.json::<JsonRpcResponse>().await?;
        
        if let Some(error) = rpc_resp.error {
            anyhow::bail!("A2A error: {} - {}", error.code, error.message);
        }
        
        let task: Task = serde_json::from_value(rpc_resp.result.unwrap())?;
        Ok(task)
    }
    
    /// Stream task updates via SSE
    pub async fn stream_task(
        &self, 
        task_id: &str
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<Task>>> {
        let url = format!("{}/tasks/{}/stream", self.base_url, task_id);
        let resp = self.http.get(&url).send().await?;
        
        let stream = resp.bytes_stream().map(|result| {
            result
                .map_err(anyhow::Error::from)
                .and_then(|bytes| {
                    let text = String::from_utf8(bytes.to_vec())?;
                    // Parse SSE format
                    if text.starts_with("data: ") {
                        let json = &text[6..];
                        let task: Task = serde_json::from_str(json)?;
                        Ok(task)
                    } else {
                        anyhow::bail!("Invalid SSE format")
                    }
                })
        });
        
        Ok(stream)
    }
    
    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str) -> anyhow::Result<()> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "tasks/cancel".to_string(),
            params: serde_json::json!({
                "taskId": task_id
            }),
        };
        
        self.http
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?;
            
        Ok(())
    }
}
```

### Kubernetes Operator (kube-rs)

```rust
// crates/k8s-operator/src/crds.rs

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AgentTask CRD
#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "metaagent.io",
    version = "v1alpha1",
    kind = "AgentTask",
    namespaced,
    status = "AgentTaskStatus",
    printcolumn = r#"{"name":"Phase", "type":"string", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Error", "type":"number", "jsonPath":".status.currentError"}"#
)]
pub struct AgentTaskSpec {
    pub description: String,
    pub controller: ControllerConfig,
    pub test_generator: TestGeneratorConfig,
    pub resource_quota: ResourceQuota,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ControllerConfig {
    pub task_weight: f32,
    pub context_weight: f32,
    pub prediction_weight: f32,
    pub error_threshold: f32,
    pub max_iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestGeneratorConfig {
    pub format: String, // "gherkin"
    pub framework: String, // "behave"
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceQuota {
    pub max_agents: u32,
    pub max_memory: String,
    pub max_cpu: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentTaskStatus {
    pub phase: String,
    pub iteration: u32,
    pub current_error: f32,
    pub tests_total: u32,
    pub tests_passed: u32,
    pub agents: Vec<AgentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentStatus {
    pub name: String,
    pub status: String,
}

/// Agent CRD
#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "metaagent.io",
    version = "v1alpha1",
    kind = "Agent",
    namespaced,
    status = "AgentCRDStatus"
)]
pub struct AgentSpec {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub model: ModelConfig,
    pub system_prompt: String,
    #[serde(default)]
    pub a2a: Option<A2AConfig>,
    #[serde(default)]
    pub a2ui: Option<A2UIConfig>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    pub resources: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelConfig {
    pub provider: String,
    pub name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct A2AConfig {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct A2UIConfig {
    pub enabled: bool,
    pub port: u16,
    pub catalog_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceRequirements {
    pub limits: ResourceLimits,
    pub requests: Option<ResourceLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceLimits {
    pub memory: String,
    pub cpu: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentCRDStatus {
    pub phase: String,
    pub registered: bool,
    pub registry_id: Option<String>,
}
```

```rust
// crates/k8s-operator/src/main.rs

use futures::StreamExt;
use kube::{
    api::{Api, ListParams, PostParams},
    runtime::controller::{Action, Controller},
    Client, ResourceExt,
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, instrument};

mod crds;
mod controllers;

use crds::{AgentTask, Agent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("k8s_operator=debug,kube=info")
        .json()
        .init();

    let client = Client::try_default().await?;
    
    // Start AgentTask controller
    let tasks: Api<AgentTask> = Api::all(client.clone());
    let agents: Api<Agent> = Api::all(client.clone());
    
    info!("Starting Meta-Agent Kubernetes Operator");
    
    let task_controller = Controller::new(tasks, ListParams::default())
        .run(
            controllers::agent_task::reconcile,
            controllers::agent_task::error_policy,
            Arc::new(controllers::agent_task::Context::new(client.clone())),
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled AgentTask: {:?}", o),
                Err(e) => error!("Reconcile error: {:?}", e),
            }
        });
    
    let agent_controller = Controller::new(agents, ListParams::default())
        .run(
            controllers::agent::reconcile,
            controllers::agent::error_policy,
            Arc::new(controllers::agent::Context::new(client.clone())),
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled Agent: {:?}", o),
                Err(e) => error!("Reconcile error: {:?}", e),
            }
        });
    
    // Run both controllers concurrently
    tokio::join!(task_controller, agent_controller);
    
    Ok(())
}
```

### Dockerfile (Multi-stage Build)

```dockerfile
# docker/Dockerfile.tcp-controller

# Build stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --package tcp-controller

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tcp-controller /usr/local/bin/

ENV RUST_LOG=tcp_controller=info

EXPOSE 8080

CMD ["tcp-controller"]
```

### Makefile for Rust

```makefile
# Makefile

.PHONY: build test lint fmt check docker-build

# Build all crates
build:
	cargo build --release

# Run tests
test:
	cargo test --workspace

# Run lints
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Check formatting and lints
check: fmt lint
	cargo check --workspace

# Build Docker images
docker-build:
	docker build -f docker/Dockerfile.tcp-controller -t metaagent/tcp-controller:latest .
	docker build -f docker/Dockerfile.agent-registry -t metaagent/agent-registry:latest .
	docker build -f docker/Dockerfile.agent-runtime -t metaagent/agent-runtime:latest .
	docker build -f docker/Dockerfile.k8s-operator -t metaagent/k8s-operator:latest .

# Run locally
run-tcp-controller:
	cargo run --package tcp-controller

run-registry:
	cargo run --package agent-registry

# Generate CRD manifests
generate-crds:
	cargo run --package k8s-operator -- generate-crds > helm/meta-agent/crds/
```
