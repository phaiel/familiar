//! Impl module for physics_hint_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for PhysicsHintResponse

// Trait impl: From
impl From < crate :: types :: RawPhysicsHint > for PhysicsHintResponse { fn from (hint : crate :: types :: RawPhysicsHint) -> Self { Self { valence : hint . valence , arousal : hint . arousal , significance : hint . significance , clarity : hint . clarity , intrusiveness : hint . intrusiveness , volatility : hint . volatility , } } }

