use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime context for CEL evaluation during routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContext {
    /// Current schema version
    pub schema_version: semver::Version,

    /// Available memory in bytes
    pub available_memory: u64,

    /// Current CPU usage (0.0 to 1.0)
    pub cpu_usage: f64,

    /// Current queue depth
    pub queue_depth: usize,

    /// Number of active jobs
    pub active_jobs: usize,

    /// Whether GPU is available
    pub has_gpu: bool,

    /// Current error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Custom properties from node configuration
    pub custom_properties: HashMap<String, serde_json::Value>,
}

impl NodeContext {
    /// Create a new context with default values
    pub fn new(schema_version: semver::Version) -> Self {
        Self {
            schema_version,
            available_memory: 0,
            cpu_usage: 0.0,
            queue_depth: 0,
            active_jobs: 0,
            has_gpu: false,
            error_rate: 0.0,
            custom_properties: HashMap::new(),
        }
    }

    /// Get the current context (would integrate with monitoring systems)
    pub fn current() -> Self {
        // TODO: Integrate with actual monitoring/metrics systems
        Self::new(semver::Version::parse("1.2.0").unwrap())
    }

    /// Convert to CEL context for evaluation
    pub fn to_cel_context(&self) -> HashMap<String, cel_interpreter::Value> {
        use cel_interpreter::Value;

        let mut context = HashMap::new();

        context.insert(
            "schema_version".to_string(),
            Value::String(self.schema_version.to_string())
        );

        context.insert(
            "available_memory".to_string(),
            Value::Int(self.available_memory as i64)
        );

        context.insert(
            "cpu_usage".to_string(),
            Value::Float(self.cpu_usage)
        );

        context.insert(
            "queue_depth".to_string(),
            Value::Int(self.queue_depth as i64)
        );

        context.insert(
            "active_jobs".to_string(),
            Value::Int(self.active_jobs as i64)
        );

        context.insert(
            "has_gpu".to_string(),
            Value::Bool(self.has_gpu)
        );

        context.insert(
            "error_rate".to_string(),
            Value::Float(self.error_rate)
        );

        // Add custom properties
        for (key, value) in &self.custom_properties {
            match value {
                serde_json::Value::String(s) => {
                    context.insert(key.clone(), Value::String(s.clone()));
                }
                serde_json::Value::Number(n) if n.is_i64() => {
                    context.insert(key.clone(), Value::Int(n.as_i64().unwrap()));
                }
                serde_json::Value::Number(n) if n.is_f64() => {
                    context.insert(key.clone(), Value::Float(n.as_f64().unwrap()));
                }
                serde_json::Value::Bool(b) => {
                    context.insert(key.clone(), Value::Bool(*b));
                }
                _ => {
                    // Convert to string for other types
                    context.insert(key.clone(), Value::String(value.to_string()));
                }
            }
        }

        context
    }
}
