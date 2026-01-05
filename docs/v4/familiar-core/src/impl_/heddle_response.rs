//! Impl module for heddle_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for HeddleResponse

// Methods: intent, is_query, is_log, query_type, query_target, validate, from_json
impl HeddleResponse { # [doc = " Get the message intent"] pub fn intent (& self) -> MessageIntent { self . message_intent . intent } # [doc = " Check if this is a query/lookup type message"] pub fn is_query (& self) -> bool { self . message_intent . intent . expects_response () } # [doc = " Check if this is a logging type message"] pub fn is_log (& self) -> bool { self . message_intent . intent . stores_data () } # [doc = " Get query type if this is a QUERY intent"] pub fn query_type (& self) -> Option < QueryType > { self . message_intent . query_type } # [doc = " Get query target if this is a QUERY intent"] pub fn query_target (& self) -> Option < & QueryTarget > { self . message_intent . query_target . as_ref () } # [doc = " Validate all weave units and convert to proper WeaveUnit types"] # [doc = " Returns (weave_units, physics_hints) - physics are separate because they go to spawned entities"] pub fn validate (self) -> Result < (Vec < WeaveUnit > , Vec < Option < RawPhysicsHint > > , RawMessageIntent) , String > { let mut units = Vec :: new () ; let mut physics = Vec :: new () ; for (idx , raw) in self . weave_units . into_iter () . enumerate () { let (unit , phys) = raw . validate (idx) ? ; units . push (unit) ; physics . push (phys) ; } if units . is_empty () && self . message_intent . intent == MessageIntent :: Log { return Err ("No weave_units in Heddle response for LOG intent" . to_string ()) ; } Ok ((units , physics , self . message_intent)) } # [doc = " Parse from JSON string"] pub fn from_json (json : & str) -> Result < Self , String > { let json = extract_json (json) ; serde_json :: from_str (json) . map_err (| e | format ! ("Failed to parse Heddle response: {}" , e)) } }

// Methods: example_json
impl HeddleResponse { # [doc = " Generate a simplified JSON example for LLM prompts"] # [doc = " Only MOMENT, PULSE, INTENT are valid entity types for LOG"] pub fn example_json () -> & 'static str { r#"For mixed LOG and QUERY messages:
{
  "message_intent": { "intent": "LOG", "confidence": 0.7 },
  "weave_units": [
    {
      "content": "meeting with someone",
      "purpose": "LOG",
      "primary_thread": "user",
      "secondary_threads": ["person name"],
      "temporal_marker": "1pm",
      "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }],
      "physics_hint": { "valence": 0.0, "arousal": 0.5, "clarity": 1.0 }
    },
    {
      "content": "feeling some way",
      "purpose": "LOG",
      "primary_thread": "user",
      "secondary_threads": [],
      "temporal_marker": "today",
      "classifications": [{ "entity_type": "PULSE", "weight": 1.0 }],
      "physics_hint": { "valence": -0.3, "arousal": 0.7, "clarity": 1.0 }
    },
    {
      "content": "when is next meeting",
      "purpose": "QUERY",
      "primary_thread": "user",
      "secondary_threads": ["meeting"],
      "temporal_marker": null,
      "classifications": [{ "entity_type": "MOMENT", "weight": 1.0 }]
    }
  ]
}

PURPOSE (per weave_unit):
- LOG: Recording information - spawns entities
- QUERY: Asking for information - no entity spawn
- COMMAND: Requesting an action - no entity spawn
- INFER: Seeking insight - no entity spawn
- REFERENCE: Looking up existing data - no entity spawn

ENTITY TYPES (for LOG purpose):
- MOMENT: A discrete event/action
- PULSE: An internal state/feeling
- INTENT: A future/planned action

message_intent is the DOMINANT purpose across all units.
Each weave_unit has its OWN purpose.

⚠️ temporal_marker examples: "today", "tonight", "1pm", "tomorrow", "yesterday", "now", "this morning"
⚠️ EVERY weave_unit should have temporal_marker if ANY time word exists
⚠️ Only LOG purpose units need physics_hint"# } }

