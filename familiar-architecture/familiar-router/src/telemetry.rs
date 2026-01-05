//! Telemetry provider interface for real-time system state

use async_trait::async_trait;
use serde::Serialize;

/// Trait for providing real-time telemetry data to the router
#[async_trait]
pub trait TelemetryProvider: Send + Sync + std::fmt::Debug {
    /// Get a snapshot of current system telemetry
    async fn get_snapshot(&self) -> anyhow::Result<serde_json::Value>;

    /// Get telemetry for a specific node
    async fn get_node_telemetry(&self, node_id: &str) -> anyhow::Result<NodeTelemetry>;

    /// Get telemetry for a specific system
    async fn get_system_telemetry(&self, system_id: &str) -> anyhow::Result<SystemTelemetry>;
}

/// Node-specific telemetry data
#[derive(Debug, Clone, Serialize)]
pub struct NodeTelemetry {
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

    /// Network latency to this node in milliseconds
    pub network_latency_ms: Option<f64>,

    /// Node health status
    pub health_status: String,
}

/// System-specific telemetry data
#[derive(Debug, Clone, Serialize)]
pub struct SystemTelemetry {
    /// Requests per second
    pub requests_per_second: f64,

    /// Average response latency in milliseconds
    pub average_latency_ms: f64,

    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Current number of active requests
    pub active_requests: u32,

    /// System health status
    pub health_status: String,
}

/// Mock telemetry provider for testing and development
#[derive(Debug)]
pub struct MockTelemetryProvider;

#[async_trait]
impl TelemetryProvider for MockTelemetryProvider {
    async fn get_snapshot(&self) -> anyhow::Result<serde_json::Value> {
        // Return mock telemetry data
        let telemetry = serde_json::json!({
            "nodes": {
                "node-1": {
                    "cpu_percent": 45.0,
                    "memory_percent": 60.0,
                    "active_tasks": 5,
                    "queue_depth": 2,
                    "health": "healthy"
                },
                "node-2": {
                    "cpu_percent": 80.0,
                    "memory_percent": 75.0,
                    "active_tasks": 12,
                    "queue_depth": 8,
                    "health": "degraded"
                }
            },
            "systems": {
                "fates-gate": {
                    "requests_per_second": 150.0,
                    "average_latency_ms": 45.0,
                    "error_rate": 0.02,
                    "active_requests": 25,
                    "health": "healthy"
                }
            }
        });

        Ok(telemetry)
    }

    async fn get_node_telemetry(&self, node_id: &str) -> anyhow::Result<NodeTelemetry> {
        // Return mock data based on node_id
        let telemetry = match node_id {
            "node-1" => NodeTelemetry {
                cpu_percent: 45.0,
                memory_percent: 60.0,
                gpu_memory_percent: None,
                active_tasks: 5,
                queue_depth: 2,
                network_latency_ms: Some(5.0),
                health_status: "healthy".to_string(),
            },
            "node-2" => NodeTelemetry {
                cpu_percent: 80.0,
                memory_percent: 75.0,
                gpu_memory_percent: Some(90.0),
                active_tasks: 12,
                queue_depth: 8,
                network_latency_ms: Some(15.0),
                health_status: "degraded".to_string(),
            },
            _ => NodeTelemetry {
                cpu_percent: 50.0,
                memory_percent: 50.0,
                gpu_memory_percent: None,
                active_tasks: 3,
                queue_depth: 1,
                network_latency_ms: Some(10.0),
                health_status: "healthy".to_string(),
            },
        };

        Ok(telemetry)
    }

    async fn get_system_telemetry(&self, system_id: &str) -> anyhow::Result<SystemTelemetry> {
        // Return mock data based on system_id
        let telemetry = match system_id {
            "fates-gate" => SystemTelemetry {
                requests_per_second: 150.0,
                average_latency_ms: 45.0,
                error_rate: 0.02,
                active_requests: 25,
                health_status: "healthy".to_string(),
            },
            "classifier-system" => SystemTelemetry {
                requests_per_second: 75.0,
                average_latency_ms: 120.0,
                error_rate: 0.05,
                active_requests: 15,
                health_status: "healthy".to_string(),
            },
            _ => SystemTelemetry {
                requests_per_second: 50.0,
                average_latency_ms: 80.0,
                error_rate: 0.01,
                active_requests: 8,
                health_status: "healthy".to_string(),
            },
        };

        Ok(telemetry)
    }
}
