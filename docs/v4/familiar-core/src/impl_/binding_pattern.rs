//! Impl module for binding_pattern types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for BindingPattern

// Methods: common_patterns
impl BindingPattern { # [doc = " Get common binding patterns"] pub fn common_patterns () -> Vec < Self > { vec ! [Self { name : "causal_because" . to_string () , binding_type : BindingType :: Causal , pattern : r"because|since|as a result|therefore|so that" . to_string () , confidence_boost : 0.3 , } , Self { name : "temporal_sequence" . to_string () , binding_type : BindingType :: Temporal , pattern : r"then|after|before|later|earlier|next" . to_string () , confidence_boost : 0.25 , } , Self { name : "contrast" . to_string () , binding_type : BindingType :: Contrastive , pattern : r"but|however|unlike|instead|whereas" . to_string () , confidence_boost : 0.3 , } , Self { name : "analogy" . to_string () , binding_type : BindingType :: Analogical , pattern : r"like|reminds me of|similar to|same as" . to_string () , confidence_boost : 0.35 , } ,] } }

