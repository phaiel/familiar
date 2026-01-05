//! Course Details Component
//!
//! DEPRECATED: Processing metadata now lives on ShuttleDetails.
//! This module re-exports ShuttleDetails as CourseDetails for backward compatibility.
//!
//! Course-Thread Architecture:
//! - Processing metadata (provider, model, latency) belongs on the SHUTTLE
//! - Session metadata (language, theme, tags) belongs on the COURSE

use crate::ShuttleDetails;

/// DEPRECATED: Use ShuttleDetails instead
/// 
/// This is a re-export for backward compatibility.
/// Processing metadata should be on the Shuttle, not the Course.
pub type CourseDetails = ShuttleDetails;
