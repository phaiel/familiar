//! AI Observation response components
//!
//! Supports both legacy single-pass and new 3-phase pipeline.

use serde::{Deserialize, Serialize};
use crate::primitives::TokenUsage;
use crate::config::{AIProvider, ModelConfig};
use crate::components::{WeaveUnit, Conversation};
use crate::types::{
    HeddleResponse, RawPhysicsHint, RawMessageIntent, MessageIntent, QueryType, QueryTarget,
    RawSegment, SegmentClassification,
};

/// Metadata about an AI completion response
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResponseMetadata {
    pub provider: AIProvider,
    pub model_id: String,
    pub model_name: String,
    pub usage: Option<TokenUsage>,
    pub latency_ms: u64,
    pub request_id: Option<String>,
}

impl ResponseMetadata {
    pub fn new(provider: AIProvider, model: &ModelConfig, latency_ms: u64) -> Self {
        Self {
            provider,
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            usage: None,
            latency_ms,
            request_id: None,
        }
    }

    pub fn from_model_info(provider: AIProvider, model_id: &str, model_name: &str, latency_ms: u64) -> Self {
        Self {
            provider,
            model_id: model_id.to_string(),
            model_name: model_name.to_string(),
            usage: None,
            latency_ms,
            request_id: None,
        }
    }

    pub fn with_usage(mut self, usage: TokenUsage) -> Self { self.usage = Some(usage); self }
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self { self.request_id = Some(id.into()); self }
}

/// Debug info about the LLM request
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LlmRequestDebug {
    pub system_prompt: String,
    pub user_prompt: String,
}

/// A complete observation response with 3-phase pipeline support
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ObservationResponse {
    // === PHASE 1: SEGMENTATION (agnostic) ===
    /// Raw segments from Phase 1 (no classification)
    #[serde(default)]
    pub segments: Vec<RawSegment>,
    
    // === PHASE 2: MESSAGE INTENT ===
    /// Classified intent of the message (LOG, QUERY, INFER, etc.)
    pub message_intent: RawMessageIntent,
    
    // === PHASE 3: CONTENT CLASSIFICATION (only for LOG) ===
    /// Classifications for each segment (only populated if LOG intent)
    #[serde(default)]
    pub classifications: Vec<SegmentClassification>,
    
    // === LEGACY FIELDS (for backward compatibility) ===
    /// Hydrated weave units (segments + classifications combined)
    #[serde(default)]
    pub weave_units: Vec<WeaveUnit>,
    /// Physics hints for spawned entities
    #[serde(default)]
    pub physics_hints: Vec<Option<RawPhysicsHint>>,
    
    // === METADATA ===
    pub metadata: ResponseMetadata,
    /// Raw JSON response from LLM (exactly what came back)
    pub raw_llm_response: Option<String>,
    /// Debug: what was sent to the LLM
    pub llm_request: Option<LlmRequestDebug>,
}

impl ObservationResponse {
    /// Create from legacy single-pass HeddleResponse (backward compatible)
    pub fn from_heddle_response(heddle_response: HeddleResponse, metadata: ResponseMetadata) -> Result<Self, String> {
        let (weave_units, physics_hints, message_intent) = heddle_response.validate()?;
        Ok(Self { 
            segments: vec![], // Legacy mode: no separate segments
            message_intent,
            classifications: vec![], // Legacy mode: classifications in weave_units
            weave_units, 
            physics_hints, 
            metadata, 
            raw_llm_response: None, 
            llm_request: None,
        })
    }
    
    /// Create from 3-phase pipeline results
    pub fn from_pipeline(
        segments: Vec<RawSegment>,
        message_intent: RawMessageIntent,
        classifications: Vec<SegmentClassification>,
        metadata: ResponseMetadata,
    ) -> Self {
        // Hydrate weave_units from segments + classifications
        let mut weave_units = Vec::new();
        let mut physics_hints = Vec::new();
        
        for seg in &segments {
            let idx = weave_units.len();
            let mut unit = WeaveUnit::new(idx, seg.content.clone());
            
            // Add subject as primary thread
            if let Some(ref subject) = seg.subject {
                unit = unit.with_primary_thread(subject.clone());
            }
            
            // Add mentions as secondary threads
            if !seg.mentions.is_empty() {
                unit = unit.with_secondary_threads(seg.mentions.clone());
            }
            
            // Add temporal marker
            if let Some(ref temporal) = seg.temporal {
                unit = unit.with_temporal_marker(temporal.clone());
            }
            
            // Find classification for this segment
            if let Some(class) = classifications.iter().find(|c| c.segment_index == idx) {
                let _ = unit.add_classification(class.entity_type, class.weight);
                
                // Convert physics
                let phys = class.physics.as_ref().map(|p| RawPhysicsHint {
                    valence: p.valence,
                    arousal: p.arousal,
                    significance: p.significance,
                    clarity: p.clarity,
                    intrusiveness: None,
                    volatility: None,
                });
                physics_hints.push(phys);
            } else {
                physics_hints.push(None);
            }
            
            weave_units.push(unit);
        }
        
        Self {
            segments,
            message_intent,
            classifications,
            weave_units,
            physics_hints,
            metadata,
            raw_llm_response: None,
            llm_request: None,
        }
    }
    
    /// Check if this is a query/lookup type response
    pub fn is_query(&self) -> bool {
        self.message_intent.intent.expects_response()
    }
    
    /// Check if this is a logging type response
    pub fn is_log(&self) -> bool {
        self.message_intent.intent.stores_data()
    }
    
    /// Get the message intent
    pub fn intent(&self) -> MessageIntent {
        self.message_intent.intent
    }
    
    /// Get query type if this is a QUERY intent
    pub fn query_type(&self) -> Option<QueryType> {
        self.message_intent.query_type
    }
    
    /// Get query target if this is a QUERY intent
    pub fn query_target(&self) -> Option<&QueryTarget> {
        self.message_intent.query_target.as_ref()
    }

    /// Attach the raw LLM output for debugging
    pub fn with_raw_response(mut self, raw: String) -> Self { 
        self.raw_llm_response = Some(raw); 
        self 
    }
    
    /// Attach request debug info from prompts
    pub fn with_request(mut self, system: String, user: String) -> Self { 
        self.llm_request = Some(LlmRequestDebug { system_prompt: system, user_prompt: user }); 
        self 
    }
    
    /// Attach request debug info directly from a Conversation (schema-first)
    pub fn with_conversation_debug(mut self, conversation: &Conversation) -> Self {
        self.llm_request = Some(LlmRequestDebug {
            system_prompt: conversation.system_prompt(),
            user_prompt: conversation.user_prompt(),
        });
        self
    }
    
    /// Attach both raw response and conversation debug in one call
    pub fn with_debug(self, raw_response: String, conversation: &Conversation) -> Self {
        self.with_raw_response(raw_response).with_conversation_debug(conversation)
    }
    
    pub fn unit_count(&self) -> usize { self.weave_units.len() }
    
    pub fn segment_count(&self) -> usize { self.segments.len() }
    
    /// Check if this response used the 3-phase pipeline
    pub fn is_pipeline(&self) -> bool { !self.segments.is_empty() }
    
    /// Should entity spawning happen for this response?
    pub fn should_spawn_entities(&self) -> bool {
        self.is_log() && !self.weave_units.is_empty()
    }
}

