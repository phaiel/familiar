//! Accept Invitation Handler
//!
//! Processes family invitation acceptance requests.

use crate::runtime::SharedResources;
use super::{AcceptInvitationRequest, AcceptInvitationResponse};
use sea_orm::TransactionTrait;
use tracing::{debug, info, instrument};

/// Execute accept invitation (step mode)
#[instrument(skip(resources, request), fields(invitation_code = %request.invitation_code))]
pub async fn execute(resources: &SharedResources, request: AcceptInvitationRequest) -> Result<String, String> {
    debug!("Executing accept invitation step");

    let response = execute_accept_invitation(resources, request).await?;

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// Core accept invitation logic with transaction
async fn execute_accept_invitation(
    resources: &SharedResources,
    request: AcceptInvitationRequest,
) -> Result<AcceptInvitationResponse, String> {
    let db = &resources.db;

    // Begin transaction
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // TODO: Implement actual invitation acceptance using SeaORM entities
    // - Find invitation by code
    // - Validate invitation (not expired, not used)
    // - Add user to tenant
    // - Mark invitation as used

    // Placeholder tenant_id (would come from invitation lookup)
    let tenant_id = uuid::Uuid::new_v4();

    txn.commit().await.map_err(|e| e.to_string())?;

    info!(
        invitation_code = %request.invitation_code,
        user_id = %request.user_id,
        "Invitation accepted"
    );

    Ok(AcceptInvitationResponse {
        tenant_id: tenant_id.to_string(),
        member_role: "member".to_string(),
        accepted_at: chrono::Utc::now().to_rfc3339(),
    })
}
