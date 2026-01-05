//! Routing trace definitions for audit logging

use crate::{RoutingContext, RoutingDecision};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete audit trail of a routing decision for debugging and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTrace {
    /// ID of the request that was routed
    pub request_id: String,

    /// When the routing decision was made
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// The final routing decision that was made
    pub decision: RoutingDecision,

    /// Step-by-step breakdown of the routing evaluation
    pub evaluation_steps: Vec<EvaluationStep>,

    /// Performance characteristics of the routing evaluation
    pub performance_metrics: PerformanceMetrics,

    /// Snapshot of the routing context at decision time
    pub context_snapshot: ContextSnapshot,

    /// Debugging information for development
    pub debug_info: DebugInfo,
}

impl RoutingTrace {
    /// Create a new routing trace for the given context
    pub fn new(context: &RoutingContext) -> Self {
        Self {
            request_id: context.request.id.clone(),
            timestamp: chrono::Utc::now(),
            decision: RoutingDecision::default(), // Will be set later
            evaluation_steps: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
            context_snapshot: ContextSnapshot::from_context(context),
            debug_info: DebugInfo::default(),
        }
    }

    /// Add an evaluation step to the trace
    pub fn add_step(&mut self, step: EvaluationStep) {
        self.evaluation_steps.push(step);
    }
}

/// Single step in the routing evaluation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationStep {
    /// Type of evaluation step
    pub step_type: EvaluationStepType,

    /// Human-readable description of this step
    pub description: String,

    /// When this step was executed
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Time taken for this step in milliseconds
    pub duration_ms: Option<f64>,

    /// CEL expressions evaluated in this step
    pub cel_expressions: Vec<CelExpressionResult>,

    /// Number of candidates evaluated at this step
    pub candidates_considered: Option<usize>,

    /// Number of candidates that passed this step
    pub candidates_filtered: Option<usize>,

    /// Step-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of evaluation steps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationStepType {
    ConstraintEvaluation,
    WeightCalculation,
    NodeFiltering,
    SystemSelection,
    FallbackActivation,
}

/// Result of evaluating a CEL expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelExpressionResult {
    /// The CEL expression string
    pub expression: String,

    /// Variable bindings used in evaluation
    pub context_values: HashMap<String, serde_json::Value>,

    /// Result of CEL evaluation
    pub result: serde_json::Value,

    /// Whether evaluation succeeded
    pub success: bool,

    /// Error message if evaluation failed
    pub error_message: Option<String>,
}

/// Performance characteristics of the routing evaluation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// Total evaluation time in milliseconds
    pub total_evaluation_time_ms: f64,

    /// Memory used during evaluation in KB
    pub memory_used_kb: Option<u64>,

    /// Number of nodes evaluated
    pub nodes_evaluated: usize,

    /// Number of systems evaluated
    pub systems_evaluated: usize,

    /// Number of constraints evaluated
    pub constraints_evaluated: usize,

    /// Cache hits during evaluation
    pub cache_hits: usize,

    /// Cache misses during evaluation
    pub cache_misses: usize,
}

/// Snapshot of routing context at decision time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Total nodes available
    pub node_count: usize,

    /// Total systems available
    pub system_count: usize,

    /// Request complexity score
    pub request_complexity: Option<f64>,

    /// Number of nodes with >80% utilization
    pub high_load_nodes: usize,

    /// Number of nodes marked as unhealthy
    pub unhealthy_nodes: usize,
}

impl ContextSnapshot {
    /// Create a context snapshot from a routing context
    pub fn from_context(context: &RoutingContext) -> Self {
        let high_load_nodes = context.nodes.values()
            .filter(|node| node.capacity.cpu_percent > 80.0 || node.capacity.memory_percent > 80.0)
            .count();

        let unhealthy_nodes = context.nodes.values()
            .filter(|node| !matches!(node.status, crate::context::NodeStatus::Healthy))
            .count();

        Self {
            node_count: context.nodes.len(),
            system_count: context.systems.len(),
            request_complexity: context.request.estimated_complexity,
            high_load_nodes,
            unhealthy_nodes,
        }
    }
}

/// Debugging information for development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Router version that made the decision
    pub router_version: String,

    /// Schema version used
    pub schema_version: String,

    /// Random seed used for tie-breaking
    pub random_seed: Option<u64>,

    /// Active feature flags during routing
    pub feature_flags: HashMap<String, bool>,

    /// Non-fatal warnings during routing
    pub warnings: Vec<String>,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            router_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version: "latest".to_string(),
            random_seed: None,
            feature_flags: HashMap::new(),
            warnings: Vec::new(),
        }
    }
}

impl Default for RoutingDecision {
    fn default() -> Self {
        Self {
            request_id: "unknown".to_string(),
            target_node: crate::decision::TargetNode {
                id: "unknown".to_string(),
                endpoint: "unknown".to_string(),
                location: None,
            },
            target_system: crate::decision::TargetSystem {
                id: "unknown".to_string(),
                r#type: "unknown".to_string(),
                capabilities: vec![],
            },
            priority: crate::context::Priority::Normal,
            confidence_score: 0.0,
            execution_parameters: None,
            fallback_options: None,
            routing_metadata: None,
        }
    }
}
