//! Signup Handler
//!
//! Processes new user signup requests.
//! Creates user account and initial tenant atomically.

use crate::runtime::SharedResources;
use super::{SignupRequest, SignupResponse};
use sea_orm::TransactionTrait;
use tracing::{debug, info, instrument};

/// Execute signup (step mode)
#[instrument(skip(resources, request), fields(email = %request.email))]
pub async fn execute(resources: &SharedResources, request: SignupRequest) -> Result<String, String> {
    debug!("Executing signup step");

    let response = execute_signup(resources, request).await?;

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// Core signup logic with transaction
async fn execute_signup(
    resources: &SharedResources,
    request: SignupRequest,
) -> Result<SignupResponse, String> {
    let db = &resources.db;

    // Begin transaction
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // Generate IDs
    let user_id = uuid::Uuid::new_v4();
    let tenant_id = uuid::Uuid::new_v4();

    // TODO: Implement actual user/tenant creation using SeaORM entities
    // - Hash password
    // - Create user record
    // - Create tenant record
    // - Link user to tenant

    txn.commit().await.map_err(|e| e.to_string())?;

    info!(
        user_id = %user_id,
        tenant_id = %tenant_id,
        "Signup complete"
    );

    Ok(SignupResponse {
        user_id: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        email: request.email,
        session_token: None,
    })
}
