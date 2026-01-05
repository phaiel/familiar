use crate::RouterResult;
use std::collections::HashMap;

/// Resource requirements for a task
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub memory_bytes: u64,
    pub cpu_cores: f64,
    pub gpu_memory_bytes: Option<u64>,
    pub network_bandwidth_mbps: Option<u32>,
}

/// Resource lease information for preventing OOM and resource conflicts
#[derive(Debug, Clone)]
pub struct ResourceLease {
    pub task_id: String,
    pub node_id: String,
    pub estimated_duration_ms: u64,
    pub resources_reserved: ResourceRequirements,
    pub leased_at: std::time::Instant,
    pub lease_id: String, // Unique identifier for this lease
}

/// Resource tracker to prevent over-subscription and OOM
pub struct ResourceTracker {
    node_capacities: HashMap<String, ResourceRequirements>,
    active_leases: HashMap<String, Vec<ResourceLease>>,
    lease_counter: u64, // For generating unique lease IDs
}

impl ResourceTracker {
    /// Create a new resource tracker
    pub fn new() -> Self {
        Self {
            node_capacities: HashMap::new(),
            active_leases: HashMap::new(),
            lease_counter: 0,
        }
    }

    /// Register a node's total capacity
    pub fn register_node_capacity(&mut self, node_id: &str, capacity: ResourceRequirements) {
        self.node_capacities.insert(node_id.to_string(), capacity);
    }

    /// Check if a node can accommodate additional resource requirements
    pub fn can_accommodate(&self, node_id: &str, requirements: &ResourceRequirements) -> RouterResult<bool> {
        let capacity = self.node_capacities.get(node_id)
            .ok_or_else(|| crate::RouterError::InvalidPolicy(
                format!("No capacity registered for node: {}", node_id)
            ))?;

        let active_leases = self.active_leases.get(node_id)
            .map(|leases| leases.as_slice())
            .unwrap_or(&[]);

        let total_reserved = self.calculate_total_reserved(active_leases);

        // Check if adding these requirements would exceed capacity
        let would_exceed_memory = total_reserved.memory_bytes + requirements.memory_bytes > capacity.memory_bytes;
        let would_exceed_cpu = total_reserved.cpu_cores + requirements.cpu_cores > capacity.cpu_cores;

        let would_exceed_gpu = if let (Some(req_gpu), Some(cap_gpu)) = (requirements.gpu_memory_bytes, capacity.gpu_memory_bytes) {
            total_reserved.gpu_memory_bytes.unwrap_or(0) + req_gpu > cap_gpu
        } else {
            false
        };

        Ok(!(would_exceed_memory || would_exceed_cpu || would_exceed_gpu))
    }

    /// Lease resources for a task
    pub fn lease_resources(
        &mut self,
        task_id: String,
        node_id: String,
        requirements: ResourceRequirements,
        estimated_duration_ms: u64
    ) -> RouterResult<String> {
        if !self.can_accommodate(&node_id, &requirements)? {
            return Err(crate::RouterError::InsufficientResources(
                format!("Cannot lease resources for task {} on node {}", task_id, node_id)
            ));
        }

        self.lease_counter += 1;
        let lease_id = format!("lease_{}", self.lease_counter);

        let lease = ResourceLease {
            task_id,
            node_id: node_id.clone(),
            estimated_duration_ms,
            resources_reserved: requirements,
            leased_at: std::time::Instant::now(),
            lease_id: lease_id.clone(),
        };

        self.active_leases.entry(node_id)
            .or_insert_with(Vec::new)
            .push(lease);

        Ok(lease_id)
    }

    /// Release resources when a task completes
    pub fn release_resources(&mut self, node_id: &str, lease_id: &str) -> RouterResult<()> {
        if let Some(leases) = self.active_leases.get_mut(node_id) {
            let initial_len = leases.len();
            leases.retain(|lease| lease.lease_id != lease_id);

            if leases.len() == initial_len {
                return Err(crate::RouterError::InvalidPolicy(
                    format!("Lease {} not found on node {}", lease_id, node_id)
                ));
            }
        } else {
            return Err(crate::RouterError::InvalidPolicy(
                format!("No leases found for node {}", node_id)
            ));
        }

        Ok(())
    }

    /// Clean up expired leases (tasks that didn't properly release)
    pub fn cleanup_expired_leases(&mut self) {
        let now = std::time::Instant::now();

        for leases in self.active_leases.values_mut() {
            leases.retain(|lease| {
                let elapsed = now.duration_since(lease.leased_at);
                let is_expired = elapsed.as_millis() as u64 > lease.estimated_duration_ms * 2; // 2x grace period
                if is_expired {
                    tracing::warn!("Cleaning up expired lease for task {} on node {}",
                                 lease.task_id, lease.node_id);
                }
                !is_expired
            });
        }
    }

    /// Get current utilization percentage for a node
    pub fn get_utilization(&self, node_id: &str) -> RouterResult<ResourceUtilization> {
        let capacity = self.node_capacities.get(node_id)
            .ok_or_else(|| crate::RouterError::InvalidPolicy(
                format!("No capacity registered for node: {}", node_id)
            ))?;

        let active_leases = self.active_leases.get(node_id)
            .map(|leases| leases.as_slice())
            .unwrap_or(&[]);

        let reserved = self.calculate_total_reserved(active_leases);

        Ok(ResourceUtilization {
            memory_percent: (reserved.memory_bytes as f64 / capacity.memory_bytes as f64) * 100.0,
            cpu_percent: (reserved.cpu_cores / capacity.cpu_cores) * 100.0,
            gpu_percent: match (reserved.gpu_memory_bytes, capacity.gpu_memory_bytes) {
                (Some(reserved), Some(capacity)) => Some((reserved as f64 / capacity as f64) * 100.0),
                _ => None,
            },
        })
    }

    /// Calculate total reserved resources across all active leases
    fn calculate_total_reserved(&self, leases: &[ResourceLease]) -> ResourceRequirements {
        let mut total = ResourceRequirements {
            memory_bytes: 0,
            cpu_cores: 0.0,
            gpu_memory_bytes: Some(0),
            network_bandwidth_mbps: None,
        };

        for lease in leases {
            total.memory_bytes += lease.resources_reserved.memory_bytes;
            total.cpu_cores += lease.resources_reserved.cpu_cores;

            if let (Some(total_gpu), Some(lease_gpu)) = (total.gpu_memory_bytes, lease.resources_reserved.gpu_memory_bytes) {
                total.gpu_memory_bytes = Some(total_gpu + lease_gpu);
            }
        }

        total
    }
}

/// Current resource utilization percentages
#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub memory_percent: f64,
    pub cpu_percent: f64,
    pub gpu_percent: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_tracking() {
        let mut tracker = ResourceTracker::new();

        // Register node capacity: 8Gi RAM, 4 CPU cores
        let capacity = ResourceRequirements {
            memory_bytes: 8 * 1024 * 1024 * 1024,
            cpu_cores: 4.0,
            gpu_memory_bytes: Some(8 * 1024 * 1024 * 1024),
            network_bandwidth_mbps: None,
        };
        tracker.register_node_capacity("node1", capacity);

        // Lease 2Gi RAM, 1 CPU core
        let requirements = ResourceRequirements {
            memory_bytes: 2 * 1024 * 1024 * 1024,
            cpu_cores: 1.0,
            gpu_memory_bytes: Some(2 * 1024 * 1024 * 1024),
            network_bandwidth_mbps: None,
        };

        let lease_id = tracker.lease_resources(
            "task1".to_string(),
            "node1".to_string(),
            requirements,
            30000
        ).unwrap();

        // Check utilization
        let utilization = tracker.get_utilization("node1").unwrap();
        assert_eq!(utilization.memory_percent, 25.0); // 2Gi used out of 8Gi
        assert_eq!(utilization.cpu_percent, 25.0);    // 1 core used out of 4
        assert_eq!(utilization.gpu_percent, Some(25.0)); // 2Gi used out of 8Gi

        // Release resources
        tracker.release_resources("node1", &lease_id).unwrap();

        // Utilization should be back to zero
        let utilization = tracker.get_utilization("node1").unwrap();
        assert_eq!(utilization.memory_percent, 0.0);
    }
}
