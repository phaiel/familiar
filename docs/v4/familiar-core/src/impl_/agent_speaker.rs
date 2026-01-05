//! Impl module for agent_speaker types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for AgentSpeaker

// Methods: as_str
impl AgentSpeaker { pub fn as_str (& self) -> & 'static str { match self { Self :: Concierge => "concierge" , Self :: Classifier => "classifier" , Self :: Physics => "physics" , Self :: Rag => "rag" , Self :: Memory => "memory" , Self :: TaskExecutor => "task_executor" , } } }

// Trait impl: Display
impl std :: fmt :: Display for AgentSpeaker { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

