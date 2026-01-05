//! Impl module for message_intent_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for MessageIntentResponse

// Trait impl: From
impl From < crate :: types :: RawMessageIntent > for MessageIntentResponse { fn from (raw : crate :: types :: RawMessageIntent) -> Self { Self { intent : raw . intent , confidence : raw . confidence , query_type : raw . query_type , query_target : raw . query_target , } } }

