//! Invitation and Join Request API Routes
//!
//! Handles family invitations and join requests:
//! - Create email/code invitations
//! - Accept invitations
//! - Request to join a family
//! - Review join requests

use utoipa::ToSchema;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use tracing::info;

use familiar_core::types::{
    CreateChannelInput, ChannelType,
    CreateEmailInviteInput, CreateCodeInviteInput, InvitationInfo, InviteRole,
    CreateJoinRequestInput, ReviewJoinRequestInput,
    CreateAuditLogInput,
};
use familiar_core::primitives::{UserId, TenantId, JoinRequestId, InvitationId};

use crate::state::AppState;
use super::{ErrorResponse, SuccessResponse};

// ============================================================================
// Helpers
// ============================================================================

/// Hash a token for storage
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract session token from headers
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    if let Some(cookie) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if part.starts_with("familiar_session=") {
                    return Some(part[17..].to_string());
                }
            }
        }
    }
    
    None
}

/// Get current user ID from session
async fn get_current_user_id(store: &familiar_core::infrastructure::TigerDataStore, headers: &HeaderMap) -> Option<UserId> {
    let token = extract_session_token(headers)?;
    let token_hash = hash_token(&token);
    store.validate_session(&token_hash).await.ok().flatten()
}

fn extract_ip(headers: &HeaderMap) -> Option<String> {
    headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}

fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers.get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvitationRequest {
    pub tenant_id: Uuid,
    #[serde(default)]
    pub invite_type: Option<String>, // "email" or "code"
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub max_uses: Option<i32>,
    #[serde(default)]
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInvitationRequest {
    /// For users accepting via invite code while creating account
    #[serde(default)]
    pub create_family_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationAcceptedResponse {
    pub success: bool,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/invitations - Create a new invitation (email or code)
pub async fn create_invitation_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateInvitationRequest>,
) -> impl IntoResponse {
    info!(tenant_id = %req.tenant_id, "Creating invitation");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get current user
    let user_id = match get_current_user_id(store, &headers).await {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    // Check if user is admin of tenant
    match store.is_user_admin(user_id, TenantId::from(req.tenant_id)).await {
        Ok(true) => {},
        Ok(false) => {
            return (StatusCode::FORBIDDEN, Json(ErrorResponse {
                error: "Only admins can create invitations".to_string(),
                code: "NOT_ADMIN".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    }

    let invite_type = req.invite_type.as_deref().unwrap_or("code");
    
    let invitation = if invite_type == "email" {
        // Email invitation
        let email = match req.email {
            Some(e) => e,
            None => {
                return (StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    error: "Email is required for email invitations".to_string(),
                    code: "EMAIL_REQUIRED".to_string(),
                })).into_response();
            }
        };

        let role = req.role.as_deref().map(|r| match r {
            "admin" => InviteRole::Admin,
            "guest" => InviteRole::Guest,
            _ => InviteRole::Member,
        });

        match store.create_email_invitation(CreateEmailInviteInput {
            tenant_id: req.tenant_id.into(),
            email,
            role,
            expires_in_days: req.expires_in_days.unwrap_or(7),
        }, user_id).await {
            Ok(inv) => inv,
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: e.to_string(),
                    code: "CREATE_INVITATION_FAILED".to_string(),
                })).into_response();
            }
        }
    } else {
        // Code invitation
        let role = req.role.as_deref().map(|r| match r {
            "admin" => InviteRole::Admin,
            "guest" => InviteRole::Guest,
            _ => InviteRole::Member,
        });

        match store.create_code_invitation(CreateCodeInviteInput {
            tenant_id: req.tenant_id.into(),
            role,
            max_uses: req.max_uses.unwrap_or(10),
            expires_in_days: req.expires_in_days,
        }, user_id).await {
            Ok(inv) => inv,
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: e.to_string(),
                    code: "CREATE_INVITATION_FAILED".to_string(),
                })).into_response();
            }
        }
    };

    Json(invitation).into_response()
}

/// GET /api/invitations/code/:code - Get invitation info by code
pub async fn get_invitation_by_code_handler(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    info!(code = %code, "Getting invitation by code");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get invitation
    let invitation = match store.get_invitation_by_code(&code).await {
        Ok(Some(inv)) => inv,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: "Invalid invitation code".to_string(),
                code: "INVALID_CODE".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    // Check if valid
    let is_valid = match store.is_invitation_valid(invitation.id).await {
        Ok(v) => v,
        Err(_) => false,
    };

    // Get tenant name
    let tenant_name = match store.get_tenant(invitation.tenant_id.into()).await {
        Ok(Some(t)) => t.name,
        _ => "Unknown Family".to_string(),
    };

    Json(InvitationInfo {
        id: invitation.id,
        tenant_id: invitation.tenant_id,
        tenant_name,
        role: invitation.role,
        is_valid,
    }).into_response()
}

/// POST /api/invitations/:id/accept - Accept an invitation
pub async fn accept_invitation_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(invitation_id): Path<Uuid>,
    Json(_req): Json<AcceptInvitationRequest>,
) -> impl IntoResponse {
    info!(invitation_id = %invitation_id, "Accepting invitation");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get current user (will be needed when implementation is complete)
    let _user_id = match get_current_user_id(store, &headers).await {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    // Check if invitation is valid
    let is_valid = match store.is_invitation_valid(InvitationId::from(invitation_id)).await {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    if !is_valid {
        return (StatusCode::GONE, Json(ErrorResponse {
            error: "This invitation is no longer valid".to_string(),
            code: "INVITATION_INVALID".to_string(),
        })).into_response();
    }

    // Get invitation details (need to fetch by ID - we'll use code lookup as workaround)
    // For now, we need to get invitation details another way
    // Let's add a helper to get by ID
    
    // For this implementation, we'll assume the frontend passes the code
    // In a real implementation, you'd add a get_invitation_by_id method
    
    // Get tenant info
    // Since we don't have direct ID lookup, this is a simplified flow
    // In production, add get_invitation_by_id to store
    
    return (StatusCode::NOT_IMPLEMENTED, Json(ErrorResponse {
        error: "Direct ID acceptance not yet implemented. Use code flow.".to_string(),
        code: "NOT_IMPLEMENTED".to_string(),
    })).into_response();
}

/// POST /api/join-requests - Create a join request
pub async fn create_join_request_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateJoinRequestInput>,
) -> impl IntoResponse {
    info!(tenant_id = %req.tenant_id, "Creating join request");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get current user
    let user_id = match get_current_user_id(store, &headers).await {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    // Check if already a member
    match store.is_user_member(user_id, req.tenant_id.into()).await {
        Ok(true) => {
            return (StatusCode::CONFLICT, Json(ErrorResponse {
                error: "You are already a member of this family".to_string(),
                code: "ALREADY_MEMBER".to_string(),
            })).into_response();
        }
        Ok(false) => {}
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    }

    // Create request
    match store.create_join_request(user_id, req).await {
        Ok(request) => Json(request).into_response(),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "CREATE_REQUEST_FAILED".to_string(),
            })).into_response()
        }
    }
}

/// GET /api/tenants/:tenant_id/join-requests - List pending join requests (admin only)
pub async fn list_join_requests_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> impl IntoResponse {
    info!(tenant_id = %tenant_id, "Listing join requests");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get current user
    let user_id = match get_current_user_id(store, &headers).await {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    // Check if user is admin
    match store.is_user_admin(user_id, TenantId::from(tenant_id)).await {
        Ok(true) => {}
        Ok(false) => {
            return (StatusCode::FORBIDDEN, Json(ErrorResponse {
                error: "Only admins can view join requests".to_string(),
                code: "NOT_ADMIN".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    }

    // Get requests
    match store.get_pending_join_requests(TenantId::from(tenant_id)).await {
        Ok(requests) => Json(requests).into_response(),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response()
        }
    }
}

/// POST /api/join-requests/:id/review - Approve or reject a join request (admin only)
pub async fn review_join_request_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(request_id): Path<Uuid>,
    Json(req): Json<ReviewJoinRequestInput>,
) -> impl IntoResponse {
    info!(request_id = %request_id, approved = %req.approved, "Reviewing join request");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get current user
    let user_id = match get_current_user_id(store, &headers).await {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Authentication required".to_string(),
                code: "UNAUTHORIZED".to_string(),
            })).into_response();
        }
    };

    // Review the request
    let reviewed_request = match store.review_join_request(JoinRequestId::from(request_id), user_id, req.clone()).await {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "REVIEW_FAILED".to_string(),
            })).into_response();
        }
    };

    // If approved, add user to tenant
    if req.approved {
        if let Err(e) = store.add_user_to_tenant(reviewed_request.user_id.into(), reviewed_request.tenant_id.into(), InviteRole::Member).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "ADD_MEMBER_FAILED".to_string(),
            })).into_response();
        }

        // Create personal channel for new member
        let _ = store.create_channel(CreateChannelInput {
            tenant_id: reviewed_request.tenant_id.into(),
            owner_id: None, // Will be linked to user's tenant_member
            name: "Personal".to_string(),
            description: Some("Your personal channel".to_string()),
            channel_type: Some(ChannelType::Personal),
        }).await;

        // Audit log
        let user = store.get_user(reviewed_request.user_id.into()).await.ok().flatten();
        let _ = store.create_audit_log(CreateAuditLogInput {
            user_id: Some(user_id.into()),
            user_email: user.map(|u| u.email),
            action: "join_request_approved".to_string(),
            resource_type: Some("tenant".to_string()),
            resource_id: Some(reviewed_request.tenant_id.into()),
            ip_address: extract_ip(&headers),
            user_agent: extract_user_agent(&headers),
            metadata: Some(serde_json::json!({ "new_member_id": reviewed_request.user_id })),
            success: true,
            error_message: None,
        }).await;
    }

    Json(reviewed_request).into_response()
}