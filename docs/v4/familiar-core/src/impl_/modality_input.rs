//! Impl module for modality_input types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ModalityInput

// Methods: text, modality
impl ModalityInput { pub fn text (content : impl Into < String >) -> Self { Self :: Text { content : content . into () , language : None , } } pub fn modality (& self) -> Modality { match self { Self :: Text { .. } => Modality :: Text , Self :: Audio { .. } => Modality :: Audio , Self :: Vision { .. } => Modality :: Vision , Self :: Video { .. } => Modality :: Video , } } }

