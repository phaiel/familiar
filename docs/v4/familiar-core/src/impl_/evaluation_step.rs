//! Impl module for evaluation_step types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EvaluationStep

// Methods: as_str, requires_ai, is_terminal
impl EvaluationStep { pub fn as_str (& self) -> & 'static str { match self { Self :: Loom => "LOOM" , Self :: Direct => "DIRECT" , Self :: Reject => "REJECT" , Self :: Retry => "RETRY" , Self :: Escalate => "ESCALATE" , Self :: Complete => "COMPLETE" , Self :: Skip => "SKIP" , } } # [doc = " Check if this step requires AI processing"] pub fn requires_ai (& self) -> bool { matches ! (self , Self :: Loom | Self :: Escalate) } # [doc = " Check if this step is terminal (no more processing)"] pub fn is_terminal (& self) -> bool { matches ! (self , Self :: Complete | Self :: Reject | Self :: Skip) } }

// Trait impl: Display
impl std :: fmt :: Display for EvaluationStep { fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result { write ! (f , "{}" , self . as_str ()) } }

