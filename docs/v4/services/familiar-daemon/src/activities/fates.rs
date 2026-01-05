//! Fates Activities
//!
//! Temporal activities for the Fates pipeline stages.
//! Each activity receives SharedState via closure capture.
//!
//! ## Opaque Envelope Pattern
//!
//! Activities receive `serde_json::Value` (opaque envelope) and unpack
//! to typed structs internally using ContractEnforcer + SIMD-JSON.

use crate::state::SharedState;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// TODO: Import from familiar-core once fates module is exposed
// For now, define placeholder types

/// Input to the Fates pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FatesInput {
    pub course_id: String,
    pub shuttle_id: String,
    pub content: String,
    pub sender_id: Option<String>,
    pub channel_id: Option<String>,
    pub tenant_id: Option<String>,
}

/// Output from Gate stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GateOutput {
    pub classification: String,
    pub next_stage: String,
    pub input: FatesInput,
    pub confidence: Option<f64>,
}

/// Output from Morta stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MortaOutput {
    pub segments: Vec<ContentSegment>,
    pub gate_output: GateOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentSegment {
    pub segment_type: String,
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Output from Decima stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DecimaOutput {
    pub entities: Vec<ExtractedEntity>,
    pub morta_output: MortaOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtractedEntity {
    pub entity_type: String,
    pub value: String,
    pub confidence: f64,
    pub segment_index: usize,
}

/// Output from Nona stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NonaOutput {
    pub response: String,
    pub intent: String,
    pub payload: Option<Value>,
    pub decima_output: DecimaOutput,
}

/// Complete pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PipelineOutput {
    pub status: String,
    pub stages: PipelineStages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PipelineStages {
    pub gate: GateOutput,
    pub morta: MortaOutput,
    pub decima: DecimaOutput,
    pub nona: NonaOutput,
}

// =============================================================================
// Activity Implementations
// =============================================================================

/// Gate activity - Classification and Routing
///
/// Determines which path the message should take.
pub async fn fates_gate_activity(
    state: SharedState,
    input: Value,
) -> Result<Value> {
    // 1. Convert to mutable Vec<u8> for SIMD-JSON (avoids String allocation)
    let mut bytes = serde_json::to_vec(&input).context("Failed to serialize input")?;

    // 2. Parse with SIMD-JSON (fast path, no validation for internal use)
    let req: FatesInput = state
        .enforcer
        .parse_mut(&mut bytes)
        .context("Failed to parse FatesInput")?;

    tracing::debug!(course_id = %req.course_id, "Gate processing");

    // 3. Execute gate logic
    // TODO: Wire up to actual familiar-core fates::gate module
    let result = GateOutput {
        classification: "default".to_string(),
        next_stage: "morta".to_string(),
        input: req,
        confidence: Some(0.95),
    };

    tracing::info!(classification = %result.classification, "Gate complete");

    // 4. Return as JSON
    serde_json::to_value(result).context("Failed to serialize GateOutput")
}

/// Morta activity - Content Segmentation
///
/// Breaks content into meaningful segments.
pub async fn fates_morta_activity(
    state: SharedState,
    input: Value,
) -> Result<Value> {
    let mut bytes = serde_json::to_vec(&input).context("Failed to serialize input")?;
    let gate_output: GateOutput = state
        .enforcer
        .parse_mut(&mut bytes)
        .context("Failed to parse GateOutput")?;

    tracing::debug!(course_id = %gate_output.input.course_id, "Morta processing");

    // TODO: Wire up to actual segmentation logic
    let result = MortaOutput {
        segments: vec![ContentSegment {
            segment_type: "text".to_string(),
            content: gate_output.input.content.clone(),
            start_pos: 0,
            end_pos: gate_output.input.content.len(),
        }],
        gate_output,
    };

    tracing::info!(segment_count = result.segments.len(), "Morta complete");

    serde_json::to_value(result).context("Failed to serialize MortaOutput")
}

/// Decima activity - Entity Extraction
///
/// Extracts entities, intents, and metadata from segments.
pub async fn fates_decima_activity(
    state: SharedState,
    input: Value,
) -> Result<Value> {
    let mut bytes = serde_json::to_vec(&input).context("Failed to serialize input")?;
    let morta_output: MortaOutput = state
        .enforcer
        .parse_mut(&mut bytes)
        .context("Failed to parse MortaOutput")?;

    tracing::debug!(
        course_id = %morta_output.gate_output.input.course_id,
        "Decima processing"
    );

    // TODO: Wire up to actual entity extraction (LLM call)
    let result = DecimaOutput {
        entities: vec![],
        morta_output,
    };

    tracing::info!(entity_count = result.entities.len(), "Decima complete");

    serde_json::to_value(result).context("Failed to serialize DecimaOutput")
}

/// Nona activity - Response Generation
///
/// Generates the final response based on extracted entities.
pub async fn fates_nona_activity(
    state: SharedState,
    input: Value,
) -> Result<Value> {
    let mut bytes = serde_json::to_vec(&input).context("Failed to serialize input")?;
    let decima_output: DecimaOutput = state
        .enforcer
        .parse_mut(&mut bytes)
        .context("Failed to parse DecimaOutput")?;

    tracing::debug!(
        course_id = %decima_output.morta_output.gate_output.input.course_id,
        "Nona processing"
    );

    // TODO: Wire up to actual response generation (LLM call)
    let result = NonaOutput {
        response: format!(
            "Processed: {}",
            decima_output.morta_output.gate_output.input.content
        ),
        intent: "acknowledge".to_string(),
        payload: None,
        decima_output,
    };

    tracing::info!(intent = %result.intent, "Nona complete");

    serde_json::to_value(result).context("Failed to serialize NonaOutput")
}

/// Pipeline activity - Full Fates Pipeline
///
/// Runs all stages in sequence as a single activity.
/// Use this for simple cases; use stage-by-stage for visibility.
pub async fn fates_pipeline_activity(
    state: SharedState,
    input: Value,
) -> Result<Value> {
    tracing::info!("Starting full Fates pipeline");

    // Run each stage in sequence
    let gate_result = fates_gate_activity(state.clone(), input).await?;
    let morta_result = fates_morta_activity(state.clone(), gate_result.clone()).await?;
    let decima_result = fates_decima_activity(state.clone(), morta_result.clone()).await?;
    let nona_result = fates_nona_activity(state.clone(), decima_result.clone()).await?;

    // Parse results for final output
    let gate: GateOutput = serde_json::from_value(gate_result).context("Failed to parse gate result")?;
    let morta: MortaOutput = serde_json::from_value(morta_result).context("Failed to parse morta result")?;
    let decima: DecimaOutput = serde_json::from_value(decima_result).context("Failed to parse decima result")?;
    let nona: NonaOutput = serde_json::from_value(nona_result).context("Failed to parse nona result")?;

    let result = PipelineOutput {
        status: "pipeline_complete".to_string(),
        stages: PipelineStages {
            gate,
            morta,
            decima,
            nona,
        },
    };

    tracing::info!("Fates pipeline complete");

    serde_json::to_value(result).context("Failed to serialize PipelineOutput")
}

