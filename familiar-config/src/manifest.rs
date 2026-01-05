use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy manifest for xtask discovery of configuration slots.
/// This allows xtask to validate that schema config:// URIs point to real parameters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyManifest {
    pub version: String,
    pub config_keys: HashMap<String, ConfigKeyInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigKeyInfo {
    pub path: String,
    pub value_type: String,
    pub default_value: serde_json::Value,
    pub description: String,
}

impl GlobalConfig {
    /// Generate a policy manifest containing all configuration keys
    pub fn generate_manifest() -> PolicyManifest {
        let mut config_keys = HashMap::new();
        
        // Generate manifest entries for each config section
        Self::add_node_keys(&mut config_keys);
        Self::add_system_keys(&mut config_keys);
        Self::add_observability_keys(&mut config_keys);
        Self::add_infra_keys(&mut config_keys);
        Self::add_pgo_keys(&mut config_keys);
        
        PolicyManifest {
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_keys,
        }
    }
    
    fn add_node_keys(keys: &mut HashMap<String, ConfigKeyInfo>) {
        let defaults = NodeResources::default();
        
        // Familiar Daemon
        Self::add_key(keys, "nodes.familiar_daemon.resources.cpu", "string", 
                     serde_json::to_value(&defaults.familiar_daemon.resources.cpu).unwrap(),
                     "CPU allocation for familiar-daemon node");
        Self::add_key(keys, "nodes.familiar_daemon.resources.memory", "string",
                     serde_json::to_value(&defaults.familiar_daemon.resources.memory).unwrap(),
                     "Memory allocation for familiar-daemon node");
        Self::add_key(keys, "nodes.familiar_daemon.resources.storage", "string",
                     serde_json::to_value(&defaults.familiar_daemon.resources.storage).unwrap(),
                     "Storage allocation for familiar-daemon node");
        Self::add_key(keys, "nodes.familiar_daemon.constraints.memory_threshold", "integer",
                     serde_json::to_value(&defaults.familiar_daemon.constraints.memory_threshold).unwrap(),
                     "Memory threshold for familiar-daemon health checks");
        Self::add_key(keys, "nodes.familiar_daemon.constraints.cpu_threshold", "number",
                     serde_json::to_value(&defaults.familiar_daemon.constraints.cpu_threshold).unwrap(),
                     "CPU threshold for familiar-daemon health checks");
        Self::add_key(keys, "nodes.familiar_daemon.constraints.queue_depth_threshold", "integer",
                     serde_json::to_value(&defaults.familiar_daemon.constraints.queue_depth_threshold).unwrap(),
                     "Queue depth threshold for familiar-daemon health checks");
        Self::add_key(keys, "nodes.familiar_daemon.constraints.error_rate_threshold", "number",
                     serde_json::to_value(&defaults.familiar_daemon.constraints.error_rate_threshold).unwrap(),
                     "Error rate threshold for familiar-daemon health checks");
        
        // Familiar Worker
        Self::add_key(keys, "nodes.familiar_worker.resources.cpu", "string",
                     serde_json::to_value(&defaults.familiar_worker.resources.cpu).unwrap(),
                     "CPU allocation for familiar-worker node");
        Self::add_key(keys, "nodes.familiar_worker.resources.memory", "string",
                     serde_json::to_value(&defaults.familiar_worker.resources.memory).unwrap(),
                     "Memory allocation for familiar-worker node");
        Self::add_key(keys, "nodes.familiar_worker.resources.storage", "string",
                     serde_json::to_value(&defaults.familiar_worker.resources.storage).unwrap(),
                     "Storage allocation for familiar-worker node");
        Self::add_key(keys, "nodes.familiar_worker.constraints.memory_threshold", "integer",
                     serde_json::to_value(&defaults.familiar_worker.constraints.memory_threshold).unwrap(),
                     "Memory threshold for familiar-worker health checks");
        Self::add_key(keys, "nodes.familiar_worker.constraints.cpu_threshold", "number",
                     serde_json::to_value(&defaults.familiar_worker.constraints.cpu_threshold).unwrap(),
                     "CPU threshold for familiar-worker health checks");
        
        // Classifier
        Self::add_key(keys, "nodes.classifier.resources.cpu", "string",
                     serde_json::to_value(&defaults.classifier.resources.cpu).unwrap(),
                     "CPU allocation for classifier node");
        Self::add_key(keys, "nodes.classifier.resources.memory", "string",
                     serde_json::to_value(&defaults.classifier.resources.memory).unwrap(),
                     "Memory allocation for classifier node");
        Self::add_key(keys, "nodes.classifier.resources.gpu", "integer",
                     serde_json::to_value(&defaults.classifier.resources.gpu).unwrap(),
                     "GPU allocation for classifier node");
        Self::add_key(keys, "nodes.classifier.constraints.memory_threshold", "integer",
                     serde_json::to_value(&defaults.classifier.constraints.memory_threshold).unwrap(),
                     "Memory threshold for classifier health checks");
    }
    
    fn add_system_keys(keys: &mut HashMap<String, ConfigKeyInfo>) {
        let defaults = SystemConfig::default();
        
        // Fates Gate
        Self::add_key(keys, "systems.fates_gate.timeouts.weave", "duration",
                     serde_json::to_value(&defaults.fates_gate.timeouts.weave).unwrap(),
                     "Timeout for FatesGate weave operations");
        Self::add_key(keys, "systems.fates_gate.timeouts.search", "duration",
                     serde_json::to_value(&defaults.fates_gate.timeouts.search).unwrap(),
                     "Timeout for FatesGate search operations");
        Self::add_key(keys, "systems.fates_gate.timeouts.classify", "duration",
                     serde_json::to_value(&defaults.fates_gate.timeouts.classify).unwrap(),
                     "Timeout for FatesGate classify operations");
        Self::add_key(keys, "systems.fates_gate.retries.weave", "integer",
                     serde_json::to_value(&defaults.fates_gate.retries.weave).unwrap(),
                     "Retry count for FatesGate weave operations");
        Self::add_key(keys, "systems.fates_gate.retries.search", "integer",
                     serde_json::to_value(&defaults.fates_gate.retries.search).unwrap(),
                     "Retry count for FatesGate search operations");
        Self::add_key(keys, "systems.fates_gate.retries.classify", "integer",
                     serde_json::to_value(&defaults.fates_gate.retries.classify).unwrap(),
                     "Retry count for FatesGate classify operations");
        
        // Classifier System
        Self::add_key(keys, "systems.classifier_system.timeouts.classification", "duration",
                     serde_json::to_value(&defaults.classifier_system.timeouts.classification).unwrap(),
                     "Timeout for classification operations");
        Self::add_key(keys, "systems.classifier_system.timeouts.entity_segment", "duration",
                     serde_json::to_value(&defaults.classifier_system.timeouts.entity_segment).unwrap(),
                     "Timeout for entity segmentation operations");
        Self::add_key(keys, "systems.classifier_system.timeouts.purpose_classification", "duration",
                     serde_json::to_value(&defaults.classifier_system.timeouts.purpose_classification).unwrap(),
                     "Timeout for purpose classification operations");
        Self::add_key(keys, "systems.classifier_system.retries.classification", "integer",
                     serde_json::to_value(&defaults.classifier_system.retries.classification).unwrap(),
                     "Retry count for classification operations");
        Self::add_key(keys, "systems.classifier_system.retries.entity_segment", "integer",
                     serde_json::to_value(&defaults.classifier_system.retries.entity_segment).unwrap(),
                     "Retry count for entity segmentation operations");
        Self::add_key(keys, "systems.classifier_system.retries.purpose_classification", "integer",
                     serde_json::to_value(&defaults.classifier_system.retries.purpose_classification).unwrap(),
                     "Retry count for purpose classification operations");
    }
    
    fn add_observability_keys(keys: &mut HashMap<String, ConfigKeyInfo>) {
        let defaults = ObservabilityConfig::default();
        
        // Circuit Breaker
        Self::add_key(keys, "observability.circuit_breaker.failure_threshold", "integer",
                     serde_json::to_value(&defaults.circuit_breaker.failure_threshold).unwrap(),
                     "Number of failures to trigger circuit breaker");
        Self::add_key(keys, "observability.circuit_breaker.success_threshold", "integer",
                     serde_json::to_value(&defaults.circuit_breaker.success_threshold).unwrap(),
                     "Number of successes to close circuit breaker");
        Self::add_key(keys, "observability.circuit_breaker.recovery_timeout", "duration",
                     serde_json::to_value(&defaults.circuit_breaker.recovery_timeout).unwrap(),
                     "Time to wait before attempting recovery");
        
        // Load Shedding
        Self::add_key(keys, "observability.load_shedding.cpu_threshold", "number",
                     serde_json::to_value(&defaults.load_shedding.cpu_threshold).unwrap(),
                     "CPU threshold to trigger load shedding");
        Self::add_key(keys, "observability.load_shedding.memory_threshold", "number",
                     serde_json::to_value(&defaults.load_shedding.memory_threshold).unwrap(),
                     "Memory threshold to trigger load shedding");
        
        // Metrics
        Self::add_key(keys, "observability.metrics.collection_interval", "duration",
                     serde_json::to_value(&defaults.metrics.collection_interval).unwrap(),
                     "How often to collect metrics");
        
        // Health Checks
        Self::add_key(keys, "observability.health_check.liveness_period", "duration",
                     serde_json::to_value(&defaults.health_check.liveness_period).unwrap(),
                     "How often to run liveness checks");
        Self::add_key(keys, "observability.health_check.readiness_period", "duration",
                     serde_json::to_value(&defaults.health_check.readiness_period).unwrap(),
                     "How often to run readiness checks");
        
        // Resilience
        Self::add_key(keys, "observability.resilience.max_attempts", "integer",
                     serde_json::to_value(&defaults.resilience.max_attempts).unwrap(),
                     "Maximum number of retry attempts");
        Self::add_key(keys, "observability.resilience.request_timeout", "duration",
                     serde_json::to_value(&defaults.resilience.request_timeout).unwrap(),
                     "Default request timeout");
        
        // Queue Monitoring
        Self::add_key(keys, "observability.queue_monitoring.max_queue_depth", "integer",
                     serde_json::to_value(&defaults.queue_monitoring.max_queue_depth).unwrap(),
                     "Maximum acceptable queue depth");
        Self::add_key(keys, "observability.queue_monitoring.max_processing_latency", "duration",
                     serde_json::to_value(&defaults.queue_monitoring.max_processing_latency).unwrap(),
                     "Maximum acceptable processing latency");
    }
    
    fn add_infra_keys(keys: &mut HashMap<String, ConfigKeyInfo>) {
        let defaults = InfraConfig::default();
        
        // Queue retention
        Self::add_key(keys, "infra.queues.classifier_retention", "string",
                     serde_json::to_value(&defaults.queues.classifier_retention).unwrap(),
                     "Message retention period for classifier queues");
        Self::add_key(keys, "infra.queues.feature_retention", "string",
                     serde_json::to_value(&defaults.queues.feature_retention).unwrap(),
                     "Message retention period for feature queues");
        
        // Scaling defaults
        Self::add_key(keys, "infra.scaling.cooldown_period", "duration",
                     serde_json::to_value(&defaults.scaling.cooldown_period).unwrap(),
                     "Cooldown period between scaling actions");
        Self::add_key(keys, "infra.timeouts.default_activity_timeout", "duration",
                     serde_json::to_value(&defaults.timeouts.default_activity_timeout).unwrap(),
                     "Default timeout for activities");
    }
    
    fn add_key(keys: &mut HashMap<String, ConfigKeyInfo>, path: &str, value_type: &str, 
               default_value: serde_json::Value, description: &str) {
        keys.insert(path.to_string(), ConfigKeyInfo {
            path: path.to_string(),
            value_type: value_type.to_string(),
            default_value,
            description: description.to_string(),
        });
    }

    fn add_pgo_keys(keys: &mut HashMap<String, ConfigKeyInfo>) {
        let defaults = PgoConfig::default();

        Self::add_key(keys, "pgo.default_sample_count", "integer",
                     serde_json::to_value(defaults.default_sample_count).unwrap(),
                     "Default number of workload samples to use for PGO training");
        Self::add_key(keys, "pgo.default_warmup_iterations", "integer",
                     serde_json::to_value(defaults.default_warmup_iterations).unwrap(),
                     "Default number of warmup iterations before PGO profiling");
        Self::add_key(keys, "pgo.default_profile_retention_days", "integer",
                     serde_json::to_value(defaults.default_profile_retention_days).unwrap(),
                     "Default days to retain PGO profile data before expiration");
        Self::add_key(keys, "pgo.default_profile_cache_ttl_days", "integer",
                     serde_json::to_value(defaults.default_profile_cache_ttl_days).unwrap(),
                     "Default TTL for PGO profile cache in days");
        Self::add_key(keys, "pgo.default_min_improvement_percent", "number",
                     serde_json::to_value(defaults.default_min_improvement_percent).unwrap(),
                     "Default minimum performance improvement required for PGO (percentage)");
        Self::add_key(keys, "pgo.default_max_regression_percent", "number",
                     serde_json::to_value(defaults.default_max_regression_percent).unwrap(),
                     "Default maximum allowed performance regression for PGO (percentage)");
        Self::add_key(keys, "pgo.profile_data_directory", "string",
                     serde_json::to_value(&defaults.profile_data_directory).unwrap(),
                     "Directory where PGO profile data is stored");
        Self::add_key(keys, "pgo.enable_by_default", "boolean",
                     serde_json::to_value(defaults.enable_by_default).unwrap(),
                     "Whether to enable PGO by default for new systems");

        // Add physics block configuration keys
        Self::add_key(keys, "physics.block.chunk_size", "string",
                     serde_json::to_value("1h").unwrap(),
                     "TimescaleDB chunk time interval for hypertables (e.g., '1h', '1d')");
        Self::add_key(keys, "physics.block.compression_delay", "string",
                     serde_json::to_value("7d").unwrap(),
                     "Delay before compressing old data chunks (e.g., '7d', '30d')");

        // Add routing configuration keys
        Self::add_key(keys, "routing.decision_timeout_ms", "integer",
                     serde_json::to_value(5000).unwrap(),
                     "Maximum time allowed for routing decision (milliseconds)");
        Self::add_key(keys, "routing.max_routing_retries", "integer",
                     serde_json::to_value(3).unwrap(),
                     "Maximum number of routing retries on failure");

        // Add node-specific routing configuration
        Self::add_key(keys, "nodes.familiar_router.constraints.memory_threshold", "string",
                     serde_json::to_value("512Mi").unwrap(),
                     "Minimum available memory for routing node");
        Self::add_key(keys, "nodes.familiar_router.constraints.cpu_threshold", "number",
                     serde_json::to_value(0.8).unwrap(),
                     "Maximum CPU usage threshold for routing node");
        Self::add_key(keys, "nodes.familiar_router.constraints.network_latency_threshold", "integer",
                     serde_json::to_value(100).unwrap(),
                     "Maximum network latency threshold (ms) for routing node");
        Self::add_key(keys, "nodes.familiar_router.constraints.max_concurrent_decisions", "integer",
                     serde_json::to_value(1000).unwrap(),
                     "Maximum concurrent routing decisions allowed");

        Self::add_key(keys, "nodes.familiar_router.resources.cpu", "string",
                     serde_json::to_value("500m").unwrap(),
                     "CPU allocation for routing node");
        Self::add_key(keys, "nodes.familiar_router.resources.memory", "string",
                     serde_json::to_value("1Gi").unwrap(),
                     "Memory allocation for routing node");
        Self::add_key(keys, "nodes.familiar_router.resources.storage", "string",
                     serde_json::to_value("10Gi").unwrap(),
                     "Storage allocation for routing node");

        // Add queue configuration
        Self::add_key(keys, "queues.routing_queue.capacity", "integer",
                     serde_json::to_value(10000).unwrap(),
                     "Maximum capacity of routing queue");
        Self::add_key(keys, "queues.routing_queue.retention_hours", "integer",
                     serde_json::to_value(24).unwrap(),
                     "Hours to retain messages in routing queue");
        Self::add_key(keys, "queues.routing_queue.max_size_mb", "integer",
                     serde_json::to_value(1024).unwrap(),
                     "Maximum size of routing queue in MB");

        // Add technique performance configuration
        Self::add_key(keys, "techniques.routing.core_routing.expected_latency_ms", "integer",
                     serde_json::to_value(50).unwrap(),
                     "Expected latency for core routing technique");
        Self::add_key(keys, "techniques.routing.core_routing.max_concurrent_executions", "integer",
                     serde_json::to_value(100).unwrap(),
                     "Maximum concurrent executions of core routing technique");
        Self::add_key(keys, "techniques.routing.core_routing.memory_requirement_mb", "integer",
                     serde_json::to_value(128).unwrap(),
                     "Memory requirement for core routing technique");
    }
}

impl PolicyManifest {
    /// Save the manifest to a JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load manifest from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let manifest = serde_json::from_str(&json)?;
        Ok(manifest)
    }
}

#[cfg(test)]

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_manifest() {
        let manifest = GlobalConfig::generate_manifest();
        println!("Generated {} config keys", manifest.config_keys.len());
        for (key, info) in &manifest.config_keys {
            println!("  {}: {}", key, info.description);
        }
        assert!(!manifest.config_keys.is_empty());
        
        // Check that some key config slots exist
        assert!(manifest.config_keys.contains_key("nodes.familiar_daemon.resources.cpu"));
        assert!(manifest.config_keys.contains_key("systems.fates_gate.timeouts.weave"));
        assert!(manifest.config_keys.contains_key("observability.circuit_breaker.failure_threshold"));
    }
    
    #[test]
    fn test_manifest_roundtrip() {
        let original = GlobalConfig::generate_manifest();
        let temp_path = std::env::temp_dir().join("test_manifest.json");
        
        original.save_to_file(&temp_path).unwrap();
        let loaded = PolicyManifest::load_from_file(&temp_path).unwrap();
        
        assert_eq!(original.version, loaded.version);
        assert_eq!(original.config_keys.len(), loaded.config_keys.len());
        
        std::fs::remove_file(&temp_path).unwrap();
    }
}
