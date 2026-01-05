//! Impl module for consent_type types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for ConsentType

// Methods: as_str
impl ConsentType { pub fn as_str (& self) -> & 'static str { match self { Self :: TermsOfService => "terms_of_service" , Self :: PrivacyPolicy => "privacy_policy" , Self :: MarketingEmails => "marketing_emails" , Self :: AiProcessing => "ai_processing" , Self :: DataSharing => "data_sharing" , Self :: Analytics => "analytics" , } } }

