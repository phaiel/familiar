//! Routing context definitions

use serde::{Deserialize, Serialize};

/// Complete world state and telemetry snapshot used by the router for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingContext {
    /// When this routing context was captured
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// The incoming request being routed
    pub request: RoutingRequest,

    /// Current state of all available nodes
    pub nodes: std::collections::HashMap<String, NodeState>,

    /// Current state of all system instances
    pub systems: std::collections::HashMap<String, SystemState>,

    /// Current global configuration snapshot
    pub global_config: GlobalConfigSnapshot,
}

/// Incoming request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    /// Unique request identifier
    pub id: String,

    /// Tenant that owns this request
    pub tenant_id: String,

    /// Request type (weave, search, classify, etc.)
    pub r#type: String,

    /// Execution priority level
    pub priority: Priority,

    /// Estimated payload size in bytes
    pub payload_size_bytes: Option<u64>,

    /// Complexity score from 0.0 (simple) to 1.0 (complex)
    pub estimated_complexity: Option<f64>,

    /// User context information
    pub user_context: Option<UserContext>,
}

/// User context for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User tier affecting priority and resource allocation
    pub tier: UserTier,

    /// Geographic region for latency optimization
    pub region: Option<String>,
}

/// User tier levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserTier {
    Free,
    Premium,
    Enterprise,
}

/// Priority levels for request execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Current state of a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    /// Current health status
    pub status: NodeStatus,

    /// Current resource utilization
    pub capacity: NodeCapacity,

    /// Node capabilities (gpu, ml, memory-intensive, etc.)
    pub specializations: Vec<String>,

    /// Geographic location information
    pub location: Option<NodeLocation>,
}

/// Node health status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Node resource utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    /// CPU utilization percentage (0-100)
    pub cpu_percent: f64,

    /// Memory utilization percentage (0-100)
    pub memory_percent: f64,

    /// GPU memory utilization percentage (0-100)
    pub gpu_memory_percent: Option<f64>,

    /// Current number of active tasks
    pub active_tasks: u32,

    /// Current queue depth
    pub queue_depth: u32,
}

/// Node geographic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLocation {
    pub region: String,
    pub zone: String,
}

/// Current state of a system instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// System instance identifier
    pub id: String,

    /// Node where this system is running
    pub node_id: String,

    /// Current operational status
    pub status: SystemStatus,

    /// Performance metrics for the system
    pub performance_metrics: Option<SystemPerformance>,
}

/// System operational status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemStatus {
    Active,
    Idle,
    Error,
    Maintenance,
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    /// Requests per second
    pub requests_per_second: f64,

    /// Average response latency in milliseconds
    pub average_latency_ms: f64,

    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Data throughput in bytes per second
    pub throughput_bytes_per_second: u64,
}

/// Global configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfigSnapshot {
    /// Routing-specific configuration
    pub routing: Option<RoutingConfig>,

    /// Load balancing configuration
    pub load_balancing: Option<LoadBalancingConfig>,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub default_timeout_ms: u64,
    pub max_retry_attempts: u32,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub strategy: LoadBalancingStrategy,
    pub health_check_interval_ms: u64,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLoaded,
    WeightedRandom,
}
