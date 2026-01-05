//! Impl module for segment_response types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for SegmentResponse

// Trait impl: From
impl From < RawSegment > for SegmentResponse { fn from (seg : RawSegment) -> Self { Self { content : seg . content , subject : seg . subject , mentions : seg . mentions , temporal : seg . temporal , } } }

