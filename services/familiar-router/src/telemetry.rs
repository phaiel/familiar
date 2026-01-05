//! Telemetry providers for consistent routing data
//!
//! This module provides interfaces and implementations for telemetry
//! data sources used in routing decisions.

use crate::{NodeContext, RouterResult, TelemetryProvider, TelemetrySnapshot};
use std::collections::HashMap;

/// Mock telemetry provider for testing
pub struct MockTelemetryProvider {
    node_states: HashMap<String, NodeContext>,
}

impl MockTelemetryProvider {
    pub fn new() -> Self {
        Self {
            node_states: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: &str, context: NodeContext) {
        self.node_states.insert(node_id.to_string(), context);
    }
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

/// In-memory telemetry provider for development
pub struct InMemoryTelemetryProvider {
    node_states: std::sync::RwLock<HashMap<String, NodeContext>>,
}

impl InMemoryTelemetryProvider {
    pub fn new() -> Self {
        Self {
            node_states: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub fn update_node(&self, node_id: &str, context: NodeContext) {
        let mut states = self.node_states.write().unwrap();
        states.insert(node_id.to_string(), context);
    }
}

#[async_trait::async_trait]
impl TelemetryProvider for InMemoryTelemetryProvider {
    async fn get_node_context(&self, node_id: &str) -> RouterResult<NodeContext> {
        let states = self.node_states.read().unwrap();
        states.get(node_id).cloned()
            .ok_or_else(|| crate::RouterError::InvalidPolicy(
                format!("Node not found: {}", node_id)
            ))
    }

    async fn snapshot(&self) -> RouterResult<TelemetrySnapshot> {
        let states = self.node_states.read().unwrap();
        Ok(TelemetrySnapshot {
            node_states: states.clone(),
            captured_at: std::time::Instant::now(),
        })
    }
}
