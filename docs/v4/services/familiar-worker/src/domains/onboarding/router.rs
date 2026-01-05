//! Onboarding Router
//!
//! Routes onboarding requests based on input type.

use crate::runtime::SharedResources;
use super::{signup, create_family, accept_invitation, SignupRequest, CreateFamilyRequest, AcceptInvitationRequest};
use tracing::{info, warn};

/// Execute routing based on input
pub async fn execute(resources: &SharedResources, input: serde_json::Value) -> Result<String, String> {
    info!("Routing onboarding request");

    // Try to determine the request type from the input
    if input.get("email").is_some() && input.get("password").is_some() {
        // Looks like a signup request
        let request: SignupRequest = serde_json::from_value(input)
            .map_err(|e| format!("Invalid signup request: {}", e))?;
        return signup::execute(resources, request).await;
    }

    if input.get("family_name").is_some() && input.get("user_id").is_some() {
        // Looks like a create family request
        let request: CreateFamilyRequest = serde_json::from_value(input)
            .map_err(|e| format!("Invalid create family request: {}", e))?;
        return create_family::execute(resources, request).await;
    }

    if input.get("invitation_code").is_some() {
        // Looks like an accept invitation request
        let request: AcceptInvitationRequest = serde_json::from_value(input)
            .map_err(|e| format!("Invalid accept invitation request: {}", e))?;
        return accept_invitation::execute(resources, request).await;
    }

    warn!("Could not determine onboarding request type from input");
    Err("Unknown onboarding request type. Expected signup, create_family, or accept_invitation".to_string())
}
