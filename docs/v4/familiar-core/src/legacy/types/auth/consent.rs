//! Consent Types
//!
//! Types for GDPR consent management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitives::UserId;

/// Type of consent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsentType {
    TermsOfService,
    PrivacyPolicy,
    MarketingEmails,
    AiProcessing,
    DataSharing,
    Analytics,
}

impl ConsentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TermsOfService => "terms_of_service",
            Self::PrivacyPolicy => "privacy_policy",
            Self::MarketingEmails => "marketing_emails",
            Self::AiProcessing => "ai_processing",
            Self::DataSharing => "data_sharing",
            Self::Analytics => "analytics",
        }
    }
}

/// A consent record
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConsentRecord {
    pub id: Uuid,  // ConsentRecordId
    pub user_id: UserId,
    pub consent_type: ConsentType,
    pub granted: bool,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for recording consent
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordConsentInput {
    pub consent_type: ConsentType,
    pub granted: bool,
    #[serde(default)]
    pub version: Option<String>,
}

/// User's current consent status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConsentStatus {
    pub terms_of_service: Option<DateTime<Utc>>,
    pub privacy_policy: Option<DateTime<Utc>>,
    pub marketing_emails: Option<DateTime<Utc>>,
    pub ai_processing: Option<DateTime<Utc>>,
    pub data_sharing: Option<DateTime<Utc>>,
    pub analytics: Option<DateTime<Utc>>,
}

