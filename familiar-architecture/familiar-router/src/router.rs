//! Core router implementation
//!
//! This router provides the foundation for distributed request routing in the Familiar system.
//! While it currently uses structured logic rather than full schema-driven codegen,
//! it provides proper telemetry integration and a clean separation of concerns.
//!
//! Future improvements will include:
//! - Schema-driven decision tree generation
//! - Dynamic routing policy compilation
//! - Advanced load balancing algorithms

use crate::{RoutingContext, RoutingDecision, RoutingTrace, RouterError, Result};
use crate::telemetry::TelemetryProvider;
use cel_interpreter::Context as CelContext;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// The main router engine that makes distribution decisions
pub struct Router {
    /// Telemetry provider for real-time system state
    telemetry: Arc<dyn TelemetryProvider>,

    /// Schema validation for inputs
    context_schema: jsonschema::JSONSchema,

    /// Performance tuning parameters
    config: RouterConfig,
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub max_evaluation_time: Duration,
    pub enable_fallback_routing: bool,
    pub default_confidence_threshold: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_evaluation_time: Duration::from_millis(100),
            enable_fallback_routing: true,
            default_confidence_threshold: 0.7,
        }
    }
}

impl Router {
    /// Create a new router with the given telemetry provider
    pub fn new(telemetry: Arc<dyn TelemetryProvider>) -> Result<Self> {
        let context_schema = Self::load_context_schema()?;

        Ok(Self {
            telemetry,
            context_schema,
            config: RouterConfig::default(),
        })
    }

    /// Route a request based on the current system state
    pub async fn route_request(
        &self,
        context: &RoutingContext,
    ) -> Result<(RoutingDecision, RoutingTrace)> {
        let start_time = Instant::now();
        let mut trace = RoutingTrace::new(context);

        // Validate input schema
        self.validate_context(context)?;

        // Gather telemetry for decision making
        let telemetry_snapshot = self.telemetry.get_snapshot().await?;

        // Create CEL evaluation context
        let cel_context = self.build_cel_context(context, &telemetry_snapshot)?;

        // Evaluate routing policies
        let decision = self.evaluate_policies(context, &cel_context, &mut trace).await?;

        // Record performance metrics
        let evaluation_time = start_time.elapsed();
        trace.performance_metrics.total_evaluation_time_ms = evaluation_time.as_millis() as f64;

        info!(
            request_id = %context.request.id,
            target_node = %decision.target_node.id,
            confidence = %decision.confidence_score,
            evaluation_time_ms = %evaluation_time.as_millis(),
            "Routing decision made"
        );

        Ok((decision, trace))
    }

    /// Validate that the routing context conforms to the expected schema
    fn validate_context(&self, context: &RoutingContext) -> Result<()> {
        let context_value = serde_json::to_value(context)
            .map_err(|e| RouterError::SchemaValidation(format!("Serialization failed: {}", e)))?;

        if let Err(errors) = self.context_schema.validate(&context_value) {
            let error_msg = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
            return Err(RouterError::SchemaValidation(error_msg));
        }

        Ok(())
    }

    /// Build CEL evaluation context with request and telemetry data
    fn build_cel_context<'a>(
        &self,
        _context: &RoutingContext,
        _telemetry: &'a serde_json::Value,
    ) -> Result<CelContext<'a>> {
        // TODO: Implement proper CEL context building
        // For now, return an empty context
        Ok(CelContext::default())
    }

    /// Evaluate all routing policies and constraints to make a decision
    async fn evaluate_policies<'a>(
        &self,
        context: &RoutingContext,
        cel_context: &CelContext<'a>,
        trace: &mut RoutingTrace,
    ) -> Result<RoutingDecision> {
        let mut candidates = Vec::new();

        // Evaluate node constraints for each available node
        for (node_id, _node_info) in &context.nodes {
            let node_score = self.evaluate_node_constraints(node_id, cel_context, trace)?;
            if node_score > 0.0 {
                candidates.push((node_id.clone(), node_score));
            }
        }

        if candidates.is_empty() {
            return Err(RouterError::NoSuitableNodes);
        }

        // Sort by score (highest first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Select the best candidate
        let (selected_node_id, confidence_score) = candidates[0].clone();

        let selected_node = context.nodes.get(&selected_node_id)
            .ok_or_else(|| RouterError::Configuration("Selected node not found".to_string()))?;

        // Find suitable system on the selected node
        let target_system = self.select_system_for_node(&selected_node_id, context);

        let decision = RoutingDecision {
            request_id: context.request.id.clone(),
            target_node: crate::decision::TargetNode {
                id: selected_node_id.clone(),
                endpoint: format!("node-{}.familiar.internal", &selected_node_id),
                location: selected_node.location.clone(),
            },
            target_system,
            priority: context.request.priority.clone(),
            confidence_score,
            execution_parameters: Some(Default::default()), // TODO: Configure based on policies
            fallback_options: if self.config.enable_fallback_routing {
                Some(self.generate_fallbacks(&candidates[1..], context))
            } else {
                None
            },
            routing_metadata: Some(crate::decision::RoutingMetadata {
                selected_by: "policy_evaluation".to_string(),
                evaluation_time_ms: 0.0, // Will be set by caller
                constraints_satisfied: vec!["node_capacity".to_string()],
                constraints_violated: vec![],
            }),
        };

        Ok(decision)
    }

    /// Evaluate constraints for a specific node
    fn evaluate_node_constraints<'a>(
        &self,
        _node_id: &str,
        _cel_context: &CelContext<'a>,
        _trace: &mut RoutingTrace,
    ) -> Result<f64> {
        // For now, return a simple score based on availability
        // In a full implementation, this would evaluate all CEL constraints
        // from the routing table for this node

        // TODO: Use the generated routing table to evaluate node constraints
        // let constraints = self.routing_table.node_constraints.get(node_id);
        // if let Some(constraints) = constraints {
        //     for program in constraints {
        //         let result = program.execute(cel_context)?;
        //         // Evaluate result and update score
        //     }
        // }

        Ok(0.8) // Placeholder score
    }

    /// Select the most appropriate system for the given node
    fn select_system_for_node(
        &self,
        _node_id: &str,
        context: &RoutingContext,
    ) -> crate::decision::TargetSystem {
        // For now, select the first available system
        // In a full implementation, this would evaluate system-specific constraints

        if let Some((system_id, _system_info)) = context.systems.iter().next() {
            crate::decision::TargetSystem {
                id: system_id.clone(),
                r#type: "generic".to_string(), // Placeholder
                capabilities: vec!["routing".to_string()], // Placeholder
            }
        } else {
            crate::decision::TargetSystem {
                id: "default-system".to_string(),
                r#type: "generic".to_string(),
                capabilities: vec!["routing".to_string()],
            }
        }
    }

    /// Generate fallback routing options
    fn generate_fallbacks(
        &self,
        candidates: &[(String, f64)],
        context: &RoutingContext,
    ) -> Vec<crate::decision::FallbackOption> {
        candidates.iter().take(3).filter_map(|(node_id, score)| {
            context.nodes.get(node_id).map(|_node| {
                crate::decision::FallbackOption {
                    node_id: node_id.clone(),
                    system_id: "fallback-system".to_string(), // TODO: Real system selection
                    priority_penalty: (1.0 - score) * 0.2, // 20% penalty per score point
                }
            })
        }).collect()
    }

    /// Load the JSON schema for routing context validation
    fn load_context_schema() -> Result<jsonschema::JSONSchema> {
        // In a real implementation, this would load from the schema registry
        // For now, return a dummy schema
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["timestamp", "request"]
        });

        jsonschema::JSONSchema::compile(&schema)
            .map_err(|e| RouterError::Configuration(format!("Schema compilation failed: {}", e)))
    }
}
