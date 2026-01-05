//! POST /weave endpoint entity
//!
//! Creates a Course from raw user input (weave).

use serde::{Deserialize, Serialize};
use crate::types::{MessageIntent, QueryType, QueryTarget, RawSegment};
use super::multimodal::{WeaveBlock, MediaRef};

// ============================================================================
// Request Component
// ============================================================================

/// Request payload for POST /weave
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeaveRequest {
    /// The raw user input (the "weave" of thought)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weave: Option<String>,
    
    /// Multimodal blocks (replaces/augments simple text weave)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<WeaveBlock>>,
    
    /// Optional global context (the "Golden Thread")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    
    /// Context can also be multimodal
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_blocks: Option<Vec<WeaveBlock>>,
    
    /// Agent/Flow ID ("heddle" or "concierge")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// AI provider to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Model ID to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Collapse threshold for entity spawning (default: 0.7)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// API key (passed from client)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

// ============================================================================
// Response Components
// ============================================================================

/// Classified message intent (what the user is trying to do)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MessageIntentResponse {
    /// Primary intent: LOG, QUERY, INFER, REFERENCE, REFLECT, COMMAND, SOCIAL
    pub intent: MessageIntent,
    /// Confidence score 0.0-1.0
    pub confidence: f64,
    /// If QUERY: what type of query
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_type: Option<QueryType>,
    /// If QUERY: what is being queried
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_target: Option<QueryTarget>,
}

/// A raw segment from Phase 1 (no classification)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentResponse {
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default)]
    pub mentions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal: Option<String>,
}

impl From<RawSegment> for SegmentResponse {
    fn from(seg: RawSegment) -> Self {
        Self {
            content: seg.content,
            subject: seg.subject,
            mentions: seg.mentions,
            temporal: seg.temporal,
        }
    }
}

/// Response payload for POST /weave
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseResponse {
    pub success: bool,
    /// Course ID (the workflow container)
    pub course_id: String,
    /// Shuttle ID (carries the weave units)
    pub shuttle_id: String,
    /// The original user input (preserved for context)
    pub original_weave: String,
    /// Provider used
    pub provider: String,
    
    /// Media references from multimodal input
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_refs: Option<Vec<MediaRef>>,
    
    // === PHASE 1: RAW SEGMENTS (agnostic) ===
    /// Raw segments before classification (Phase 1 output)
    #[serde(default)]
    pub segments: Vec<SegmentResponse>,
    
    // === PHASE 2: MESSAGE INTENT CLASSIFICATION ===
    /// What is the user trying to do with this message?
    pub message_intent: MessageIntentResponse,
    
    // === PHASE 3: CLASSIFIED WEAVE UNITS (only for LOG) ===
    /// Number of weave units extracted (0 for queries)
    pub unit_count: usize,
    /// The classified weave units (for LOG intent)
    pub weave_units: Vec<WeaveUnitResponse>,
    /// Entities spawned from weave units (by Spawner, not LLM)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<EntityResponse>>,
    /// Processing metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Error message if failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    // === DEBUG: Pipeline visibility ===
    /// Raw prompt sent to LLM (system + user message)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_llm_request: Option<crate::components::LlmRequestDebug>,
    /// Raw JSON response from LLM (before processing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_llm_response: Option<String>,
}

impl CourseResponse {
    pub fn error(course_id: &str, shuttle_id: &str, original_weave: &str, provider: &str, message: String) -> Self {
        Self {
            success: false,
            course_id: course_id.to_string(),
            shuttle_id: shuttle_id.to_string(),
            original_weave: original_weave.to_string(),
            provider: provider.to_string(),
            media_refs: None,
            segments: vec![],
            message_intent: MessageIntentResponse {
                intent: MessageIntent::Log,
                confidence: 0.0,
                query_type: None,
                query_target: None,
            },
            unit_count: 0,
            weave_units: vec![],
            entities: None,
            metadata: None,
            error: Some(message),
            debug_llm_request: None,
            debug_llm_response: None,
        }
    }
}

impl From<crate::types::RawMessageIntent> for MessageIntentResponse {
    fn from(raw: crate::types::RawMessageIntent) -> Self {
        Self {
            intent: raw.intent,
            confidence: raw.confidence,
            query_type: raw.query_type,
            query_target: raw.query_target,
        }
    }
}

/// A weave unit in the response
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeaveUnitResponse {
    pub index: usize,
    pub content: String,
    /// Purpose of this specific weave unit (LOG, QUERY, COMMAND, INFER, REFERENCE)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// Primary thread (main subject/actor)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_thread: Option<String>,
    /// Secondary threads (other people/places/things)
    #[serde(default)]
    pub secondary_threads: Vec<String>,
    /// Temporal marker (when: absolute, relative, frequency, duration)
    /// Always included in output for debugging (even if null)
    #[serde(default)]
    pub temporal_marker: Option<String>,
    pub classifications: Vec<ClassificationResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physics_hint: Option<PhysicsHintResponse>,
    #[serde(default)]
    pub spawned_ids: Vec<String>,
}

/// Classification with weight
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassificationResponse {
    pub entity_type: String,
    pub weight: f64,
}

/// Physics hints from LLM
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsHintResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arousal: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub significance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intrusiveness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volatility: Option<f64>,
}

impl From<crate::types::RawPhysicsHint> for PhysicsHintResponse {
    fn from(hint: crate::types::RawPhysicsHint) -> Self {
        Self {
            valence: hint.valence,
            arousal: hint.arousal,
            significance: hint.significance,
            clarity: hint.clarity,
            intrusiveness: hint.intrusiveness,
            volatility: hint.volatility,
        }
    }
}

/// A spawned entity in the response
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityResponse {
    pub entity_type: String,
    pub id: String,
    pub content_preview: String,
    pub unit_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physics: Option<PhysicsResponse>,
}

/// Physics state on spawned entity
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PhysicsResponse {
    /// Position in VAE space [Valence, Arousal, Epistemic]
    pub position: [f64; 3],
    pub amplitude: f64,
    pub energy: f64,
    pub temperature: f64,
}

