//! Tenant Types
//!
//! Types for multi-tenant (family) management.

use serde::{Deserialize, Serialize};

use crate::primitives::TenantId;
use crate::types::base::SystemEntityMeta;

/// A tenant (family) in the system
/// 
/// Uses `SystemEntityMeta` because tenants are top-level entities
/// that don't belong to another tenant.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Tenant {
    /// Entity metadata (id, timestamps)
    #[serde(flatten)]
    pub meta: SystemEntityMeta<TenantId>,
    pub name: String,
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// Input for creating a new tenant
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateTenantInput {
    pub name: String,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

