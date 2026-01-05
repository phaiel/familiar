use crate::{CALEvaluator, NodeContext, ResourceTracker, RouterResult, ResourceRequirements};
use std::collections::HashMap;

/// Telemetry provider interface for consistent data
#[async_trait::async_trait]
pub trait TelemetryProvider: Send + Sync {
    async fn get_node_context(&self, node_id: &str) -> RouterResult<NodeContext>;
    async fn snapshot(&self) -> RouterResult<TelemetrySnapshot>;
}

/// Snapshot of telemetry data for consistency
#[derive(Clone)]
pub struct TelemetrySnapshot {
    pub node_states: HashMap<String, NodeContext>,
    pub captured_at: std::time::Instant,
}

/// Decision made by the router about where to send a request
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// The target queue/topic to send to
    pub queue: String,

    /// Priority level (if applicable)
    pub priority: Option<String>,

    /// Additional routing metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Resource lease ID for tracking
    pub lease_id: Option<String>,
}

/// Main router for making intelligent routing decisions
pub struct Router {
    evaluator: CALEvaluator,
    resource_tracker: ResourceTracker,
    telemetry_provider: Box<dyn TelemetryProvider>,
}

#[derive(Debug, Clone)]
struct RouteConfig {
    trigger: String,
    routing_policy: Option<String>,
    default_queue: Option<String>,
}

impl Router {
    /// Create a new router with a telemetry provider
    pub fn new(telemetry_provider: Box<dyn TelemetryProvider>) -> Self {
        Self {
            evaluator: CALEvaluator::new(),
            resource_tracker: ResourceTracker::new(),
            telemetry_provider,
        }
    }

    /// Register a node with its resource capacity
    pub fn register_node_capacity(&mut self, node_id: &str, capacity: ResourceRequirements) {
        self.resource_tracker.register_node_capacity(node_id, capacity);
    }

    /// Add a routing rule from a system configuration
    pub fn add_route(&mut self, trigger: &str, routing_policy: Option<&str>, default_queue: Option<&str>) {
        let config = RouteConfig {
            trigger: trigger.to_string(),
            routing_policy: routing_policy.map(|s| s.to_string()),
            default_queue: default_queue.map(|s| s.to_string()),
        };

        // For simplicity, we'll handle routes dynamically in the routing logic
        // In a real implementation, this would build a routing table
    }

    /// Make a routing decision for a given trigger and input
    pub async fn route(&mut self, trigger: &str, input: &serde_json::Value) -> RouterResult<RouteDecision> {
        // Get current telemetry snapshot for consistent evaluation
        let snapshot = self.telemetry_provider.snapshot().await?;

        // Find suitable nodes based on routing policies
        // This is a simplified implementation - in reality, this would
        // evaluate system routing policies from the schema

        // For now, implement basic logic: route based on input size
        let input_size = estimate_input_size(input);
        let target_node = if input_size > 10000 {
            "familiar-daemon" // High-memory node for large inputs
        } else {
            "familiar-worker" // Standard node for smaller inputs
        };

        // Check if target node can accept work
        let node_context = snapshot.node_states.get(target_node)
            .ok_or_else(|| crate::RouterError::InvalidPolicy(
                format!("No telemetry data for node: {}", target_node)
            ))?;

        // Check resource availability (simplified requirements)
        let requirements = ResourceRequirements {
            memory_bytes: 256 * 1024 * 1024, // 256Mi baseline
            cpu_cores: 0.1,
            gpu_memory_bytes: None,
            network_bandwidth_mbps: None,
        };

        if !self.resource_tracker.can_accommodate(target_node, &requirements)? {
            return Err(crate::RouterError::InsufficientResources(
                format!("Insufficient resources on node {}", target_node)
            ));
        }

        // Lease resources
        let lease_id = self.resource_tracker.lease_resources(
            format!("{}_{}", trigger, chrono::Utc::now().timestamp()),
            target_node.to_string(),
            requirements,
            30000 // 30 second estimated duration
        )?;

        // Determine queue based on node
        let queue = match target_node {
            "familiar-daemon" => "daemon-queue",
            "familiar-worker" => "worker-queue",
            "classifier" => "classifier-queue",
            _ => return Err(crate::RouterError::InvalidPolicy(
                format!("Unknown node type: {}", target_node)
            )),
        };

        Ok(RouteDecision {
            queue: queue.to_string(),
            priority: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("input_size".to_string(), serde_json::json!(input_size));
                meta.insert("target_node".to_string(), serde_json::json!(target_node));
                meta.insert("routing_strategy".to_string(), serde_json::json!("size_based"));
                meta
            },
            lease_id: Some(lease_id),
        })
    }

    /// Release resources when a task completes
    pub fn release_resources(&mut self, node_id: &str, lease_id: &str) -> RouterResult<()> {
        self.resource_tracker.release_resources(node_id, lease_id)
    }

    /// Clean up expired resource leases
    pub fn cleanup_expired_leases(&mut self) {
        self.resource_tracker.cleanup_expired_leases();
    }

    /// Get current resource utilization for a node
    pub fn get_node_utilization(&self, node_id: &str) -> RouterResult<crate::resource_tracker::ResourceUtilization> {
        self.resource_tracker.get_utilization(node_id)
    }
}

/// Estimate input size for routing decisions
fn estimate_input_size(input: &serde_json::Value) -> usize {
    // Simple estimation - in practice, this would be more sophisticated
    serde_json::to_string(input).map(|s| s.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock telemetry provider for testing
    struct MockTelemetryProvider {
        node_states: HashMap<String, NodeContext>,
    }

    #[async_trait::async_trait]
    impl TelemetryProvider for MockTelemetryProvider {
        async fn get_node_context(&self, node_id: &str) -> RouterResult<NodeContext> {
            self.node_states.get(node_id).cloned()
                .ok_or_else(|| crate::RouterError::InvalidPolicy(
                    format!("Node not found: {}", node_id)
                ))
        }

        async fn snapshot(&self) -> RouterResult<TelemetrySnapshot> {
            Ok(TelemetrySnapshot {
                node_states: self.node_states.clone(),
                captured_at: std::time::Instant::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_routing_with_resources() {
        let mut mock_telemetry = MockTelemetryProvider {
            node_states: HashMap::new(),
        };

        // Add node states
        mock_telemetry.node_states.insert(
            "familiar-daemon".to_string(),
            NodeContext {
                schema_version: semver::Version::parse("1.2.0").unwrap(),
                available_memory: 4 * 1024 * 1024 * 1024, // 4Gi
                cpu_usage: 0.5,
                queue_depth: 10,
                active_jobs: 2,
                has_gpu: false,
                error_rate: 0.01,
                custom_properties: HashMap::new(),
            }
        );

        let telemetry_provider = Box::new(mock_telemetry);
        let mut router = Router::new(telemetry_provider);

        // Register node capacity
        let capacity = ResourceRequirements {
            memory_bytes: 8 * 1024 * 1024 * 1024, // 8Gi
            cpu_cores: 4.0,
            gpu_memory_bytes: None,
            network_bandwidth_mbps: None,
        };
        router.register_node_capacity("familiar-daemon", capacity);

        // Test routing a small input
        let small_input = serde_json::json!({"data": "small"});
        let decision = router.route("test-trigger", &small_input).await.unwrap();

        assert_eq!(decision.queue, "daemon-queue");
        assert!(decision.lease_id.is_some());

        // Check utilization
        let utilization = router.get_node_utilization("familiar-daemon").unwrap();
        assert!(utilization.memory_percent > 0.0); // Should have leased some memory
    }
}
