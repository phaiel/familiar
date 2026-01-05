//! Routing decision definitions

use crate::context::{Priority, NodeLocation};
use serde::{Deserialize, Serialize};

/// The output of the routing engine specifying where and how to execute a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// ID of the request being routed
    pub request_id: String,

    /// The selected execution node
    pub target_node: TargetNode,

    /// The selected system instance to handle the request
    pub target_system: TargetSystem,

    /// Computed priority level for execution
    pub priority: Priority,

    /// Router's confidence in this decision (0.0 = random guess, 1.0 = optimal)
    pub confidence_score: f64,

    /// Runtime parameters for execution
    pub execution_parameters: Option<ExecutionParameters>,

    /// Alternative routing options if primary fails
    pub fallback_options: Option<Vec<FallbackOption>>,

    /// Additional metadata about the routing decision
    pub routing_metadata: Option<RoutingMetadata>,
}

/// Target node for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetNode {
    /// Node identifier
    pub id: String,

    /// Network endpoint for the node
    pub endpoint: String,

    /// Geographic location information
    pub location: Option<NodeLocation>,
}

/// Target system for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSystem {
    /// System instance identifier
    pub id: String,

    /// System type (FatesGate, ClassifierSystem, etc.)
    pub r#type: String,

    /// System capabilities relevant to this request
    pub capabilities: Vec<String>,
}

/// Runtime execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionParameters {
    /// Maximum execution timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Maximum number of retries allowed
    pub max_retries: Option<u32>,

    /// Target queue name for execution
    pub priority_queue: Option<String>,

    /// Resource limits for this execution
    pub resource_limits: Option<ResourceLimits>,
}

/// Resource limits for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: Option<f64>,

    /// Maximum memory in MB
    pub max_memory_mb: Option<u64>,

    /// Maximum GPU memory in MB
    pub max_gpu_memory_mb: Option<u64>,
}

/// Fallback routing option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackOption {
    /// Alternative node identifier
    pub node_id: String,

    /// Alternative system identifier
    pub system_id: String,

    /// Performance penalty vs primary option (0.0 = no penalty, 1.0 = significant penalty)
    pub priority_penalty: f64,
}

/// Routing decision metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetadata {
    /// Algorithm or strategy that selected this route
    pub selected_by: String,

    /// Time taken to make the routing decision in milliseconds
    pub evaluation_time_ms: f64,

    /// List of constraints that were satisfied
    pub constraints_satisfied: Vec<String>,

    /// List of constraints that were violated but overridden
    pub constraints_violated: Vec<String>,
}

impl Default for ExecutionParameters {
    fn default() -> Self {
        Self {
            timeout_ms: Some(30000), // 30 seconds
            max_retries: Some(3),
            priority_queue: Some("normal".to_string()),
            resource_limits: None,
        }
    }
}

impl Default for RoutingMetadata {
    fn default() -> Self {
        Self {
            selected_by: "default".to_string(),
            evaluation_time_ms: 0.0,
            constraints_satisfied: vec![],
            constraints_violated: vec![],
        }
    }
}
