//! Create Family Handler
//!
//! Processes family/tenant creation requests.

use crate::runtime::SharedResources;
use super::{CreateFamilyRequest, CreateFamilyResponse};
use sea_orm::TransactionTrait;
use tracing::{debug, info, instrument};

/// Execute create family (step mode)
#[instrument(skip(resources, request), fields(family_name = %request.family_name))]
pub async fn execute(resources: &SharedResources, request: CreateFamilyRequest) -> Result<String, String> {
    debug!("Executing create family step");

    let response = execute_create_family(resources, request).await?;

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// Core create family logic with transaction
async fn execute_create_family(
    resources: &SharedResources,
    request: CreateFamilyRequest,
) -> Result<CreateFamilyResponse, String> {
    let db = &resources.db;

    // Begin transaction
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // Generate tenant ID
    let tenant_id = uuid::Uuid::new_v4();

    // TODO: Implement actual family creation using SeaORM entities
    // - Create tenant record with family_name
    // - Link user as owner

    txn.commit().await.map_err(|e| e.to_string())?;

    info!(
        tenant_id = %tenant_id,
        family_name = %request.family_name,
        "Family created"
    );

    Ok(CreateFamilyResponse {
        tenant_id: tenant_id.to_string(),
        family_name: request.family_name,
        owner_id: request.user_id,
    })
}
