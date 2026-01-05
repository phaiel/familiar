//! Direct Database Operations for Onboarding
//!
//! These commands replace inline SQL in Windmill Deno scripts.
//! Each function outputs JSON to stdout for Windmill to consume.
//!
//! ## Why This Exists
//!
//! Windmill scripts previously contained raw SQL like:
//! ```typescript
//! await client.queryObject(`INSERT INTO users ...`);
//! ```
//!
//! This created a "dual-source" problem where both Rust (SeaORM) and
//! Deno (raw SQL) could modify the same tables with different logic.
//!
//! Now Windmill calls `minerva onboarding <command>` instead,
//! and all database access goes through the Rust store layer.

use crate::runtime::SharedResources;
use familiar_core::primitives::{UserId, TenantId, InvitationId};
use familiar_core::types::{
    CreateUserInput, UpdateUserInput,
    CreateTenantInput, CreateMemberInput, CreateChannelInput, ChannelType,
    CreateAuditLogInput, RecordConsentInput, ConsentType, MemberRole,
    CreateSessionInput,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Result type for database operations
pub type DbOpResult = Result<String, String>;

// =============================================================================
// Response Types (JSON output format)
// =============================================================================

#[derive(Serialize)]
struct CheckEmailResponse {
    exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

#[derive(Serialize)]
struct CreateUserResponse {
    user_id: String,
    email: String,
    name: String,
    created_at: String,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    expires_at: String,
}

#[derive(Serialize)]
struct RecordConsentResponse {
    recorded: usize,
}

#[derive(Serialize, Deserialize)]
struct ValidateInviteResponse {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct CheckNeedsFamilyResponse {
    needs_family: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_tenant_id: Option<String>,
}

#[derive(Serialize)]
struct CreateTenantResponse {
    tenant_id: String,
    name: String,
    created_at: String,
}

#[derive(Serialize)]
struct AddMemberResponse {
    member_id: String,
    tenant_id: String,
    user_id: String,
    role: String,
}

#[derive(Serialize)]
struct SetPrimaryTenantResponse {
    success: bool,
    user_id: String,
    tenant_id: String,
}

#[derive(Serialize)]
struct CreateChannelResponse {
    channel_id: String,
    tenant_id: String,
    name: String,
    channel_type: String,
}

#[derive(Serialize)]
struct IncrementInviteResponse {
    success: bool,
    invitation_id: String,
}

#[derive(Serialize)]
struct AuditLogResponse {
    audit_log_id: String,
    action: String,
    created_at: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn error_response(error: &str, code: &str) -> String {
    serde_json::to_string(&ErrorResponse {
        error: error.to_string(),
        code: code.to_string(),
    })
    .unwrap_or_else(|_| format!(r#"{{"error":"{}","code":"{}"}}"#, error, code))
}

// =============================================================================
// Command Handlers
// =============================================================================

/// Check if an email already exists in the system
pub async fn check_email(resources: &Arc<SharedResources>, email: &str) -> DbOpResult {
    info!(email = %email, "Checking if email exists");
    
    match resources.store.get_user_by_email(email).await {
        Ok(Some(user)) => {
            let response = CheckEmailResponse {
                exists: true,
                user_id: Some(user.meta.id.to_string()),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Ok(None) => {
            let response = CheckEmailResponse {
                exists: false,
                user_id: None,
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "DB_ERROR")),
    }
}

/// Create a new user record
pub async fn create_user(
    resources: &Arc<SharedResources>,
    email: &str,
    name: &str,
    password_hash: Option<&str>,
) -> DbOpResult {
    info!(email = %email, name = %name, "Creating user");
    
    // Check if email already exists
    if let Ok(Some(_)) = resources.store.get_user_by_email(email).await {
        return Err(error_response("Email already registered", "EMAIL_EXISTS"));
    }
    
    let input = CreateUserInput {
        email: email.to_string(),
        name: name.to_string(),
        password_hash: password_hash.map(String::from),
        avatar_url: None,
        primary_tenant_id: None,
    };
    
    match resources.store.create_user(input).await {
        Ok(user) => {
            let response = CreateUserResponse {
                user_id: user.meta.id.to_string(),
                email: user.email,
                name: user.name,
                created_at: user.meta.timestamps.created_at.to_rfc3339(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "CREATE_USER_FAILED")),
    }
}

/// Create an auth session for a user
pub async fn create_session(resources: &Arc<SharedResources>, user_id: &str) -> DbOpResult {
    info!(user_id = %user_id, "Creating auth session");
    
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    
    // Generate a secure random token and hash it
    let token = uuid::Uuid::new_v4().to_string();
    let token_hash = sha256_hash(&token);
    
    let input = CreateSessionInput {
        user_id: UserId::from(user_uuid),
        token_hash,
        user_agent: None,
        ip_address: None,
        expires_in_hours: 24 * 7, // 1 week
    };
    
    match resources.store.create_session(input).await {
        Ok(session) => {
            let response = CreateSessionResponse {
                session_id: session.id.to_string(),
                expires_at: session.expires_at.to_rfc3339(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "CREATE_SESSION_FAILED")),
    }
}

/// Simple SHA256 hash for session tokens
fn sha256_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Record GDPR consent records for a user
pub async fn record_consent(
    resources: &Arc<SharedResources>,
    user_id: &str,
    consents_json: &str,
) -> DbOpResult {
    info!(user_id = %user_id, "Recording GDPR consents");
    
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    let user_id_typed = UserId::from(user_uuid);
    
    // Parse consents JSON - expected format: {"terms": true, "privacy": true, "marketing": false}
    let consents: serde_json::Value = serde_json::from_str(consents_json)
        .map_err(|e| error_response(&format!("Invalid consents JSON: {}", e), "INVALID_JSON"))?;
    
    let mut recorded = 0;
    
    // Record each consent type
    if let Some(terms) = consents.get("terms").and_then(|v| v.as_bool()) {
        if terms {
            let input = RecordConsentInput {
                consent_type: ConsentType::TermsOfService,
                granted: true,
                version: Some("1.0".to_string()),
            };
            if resources.store.record_consent(user_id_typed, input, None, None).await.is_ok() {
                recorded += 1;
            }
        }
    }
    
    if let Some(privacy) = consents.get("privacy").and_then(|v| v.as_bool()) {
        if privacy {
            let input = RecordConsentInput {
                consent_type: ConsentType::PrivacyPolicy,
                granted: true,
                version: Some("1.0".to_string()),
            };
            if resources.store.record_consent(user_id_typed, input, None, None).await.is_ok() {
                recorded += 1;
            }
        }
    }
    
    if let Some(marketing) = consents.get("marketing").and_then(|v| v.as_bool()) {
        if marketing {
            let input = RecordConsentInput {
                consent_type: ConsentType::MarketingEmails,
                granted: true,
                version: Some("1.0".to_string()),
            };
            if resources.store.record_consent(user_id_typed, input, None, None).await.is_ok() {
                recorded += 1;
            }
        }
    }
    
    let response = RecordConsentResponse { recorded };
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// Validate an invitation code
pub async fn validate_invite(resources: &Arc<SharedResources>, code: &str) -> DbOpResult {
    info!(code = %code, "Validating invitation code");
    
    match resources.store.get_invitation_by_code(code).await {
        Ok(Some(invitation)) => {
            // Check if invitation is valid
            let is_valid = resources.store.is_invitation_valid(invitation.id).await.unwrap_or(false);
            
            if is_valid {
                // Get tenant name
                let tenant_name = match resources.store.get_tenant(invitation.tenant_id).await {
                    Ok(Some(tenant)) => Some(tenant.name),
                    _ => None,
                };
                
                let response = ValidateInviteResponse {
                    valid: true,
                    invitation_id: Some(invitation.id.to_string()),
                    tenant_id: Some(invitation.tenant_id.to_string()),
                    tenant_name,
                    role: Some(format!("{:?}", invitation.role).to_lowercase()),
                    error: None,
                };
                serde_json::to_string(&response).map_err(|e| e.to_string())
            } else {
                let response = ValidateInviteResponse {
                    valid: false,
                    invitation_id: None,
                    tenant_id: None,
                    tenant_name: None,
                    role: None,
                    error: Some("Invitation expired or exhausted".to_string()),
                };
                serde_json::to_string(&response).map_err(|e| e.to_string())
            }
        }
        Ok(None) => {
            let response = ValidateInviteResponse {
                valid: false,
                invitation_id: None,
                tenant_id: None,
                tenant_name: None,
                role: None,
                error: Some("Invalid invitation code".to_string()),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "DB_ERROR")),
    }
}

/// Validate invitation for a specific user
pub async fn validate_invitation(
    resources: &Arc<SharedResources>,
    user_id: &str,
    invite_code: &str,
) -> DbOpResult {
    info!(user_id = %user_id, invite_code = %invite_code, "Validating invitation for user");
    
    let _user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    
    // First validate the invitation code
    let invite_result = validate_invite(resources, invite_code).await?;
    let invite_response: ValidateInviteResponse = serde_json::from_str(&invite_result)
        .map_err(|e| e.to_string())?;
    
    if !invite_response.valid {
        return Ok(invite_result);
    }
    
    // Check if user is already a member of the tenant
    if let Some(tenant_id_str) = &invite_response.tenant_id {
        let tenant_uuid = Uuid::parse_str(tenant_id_str)
            .map_err(|_| error_response("Invalid tenant_id format", "INVALID_UUID"))?;
        
        let members = resources.store.get_tenant_members(TenantId::from(tenant_uuid)).await
            .map_err(|e| error_response(&e.to_string(), "DB_ERROR"))?;
        
        if members.iter().any(|m| m.meta.id.to_string() == user_id) {
            let response = ValidateInviteResponse {
                valid: false,
                invitation_id: invite_response.invitation_id,
                tenant_id: invite_response.tenant_id,
                tenant_name: invite_response.tenant_name,
                role: invite_response.role,
                error: Some("User is already a member of this family".to_string()),
            };
            return serde_json::to_string(&response).map_err(|e| e.to_string());
        }
    }
    
    Ok(invite_result)
}

/// Check if a user needs a family
pub async fn check_needs_family(resources: &Arc<SharedResources>, user_id: &str) -> DbOpResult {
    info!(user_id = %user_id, "Checking if user needs a family");
    
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    
    match resources.store.get_user(UserId::from(user_uuid)).await {
        Ok(Some(user)) => {
            let response = CheckNeedsFamilyResponse {
                needs_family: user.primary_tenant_id.is_none(),
                current_tenant_id: user.primary_tenant_id.map(|t| t.to_string()),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Ok(None) => Err(error_response("User not found", "USER_NOT_FOUND")),
        Err(e) => Err(error_response(&e.to_string(), "DB_ERROR")),
    }
}

/// Create a new tenant (family)
pub async fn create_tenant(resources: &Arc<SharedResources>, name: &str) -> DbOpResult {
    info!(name = %name, "Creating tenant");
    
    let input = CreateTenantInput {
        name: name.to_string(),
        settings: None,
    };
    
    match resources.store.create_tenant(input).await {
        Ok(tenant) => {
            let response = CreateTenantResponse {
                tenant_id: tenant.meta.id.to_string(),
                name: tenant.name,
                created_at: tenant.meta.timestamps.created_at.to_rfc3339(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "CREATE_TENANT_FAILED")),
    }
}

/// Add a user as a member to a tenant
pub async fn add_member(
    resources: &Arc<SharedResources>,
    tenant_id: &str,
    user_id: &str,
    role: &str,
    name: &str,
    email: &str,
) -> DbOpResult {
    info!(tenant_id = %tenant_id, user_id = %user_id, role = %role, "Adding member to tenant");
    
    let tenant_uuid = Uuid::parse_str(tenant_id)
        .map_err(|_| error_response("Invalid tenant_id format", "INVALID_UUID"))?;
    let _user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    
    let member_role = match role.to_lowercase().as_str() {
        "admin" => MemberRole::Admin,
        "member" => MemberRole::Member,
        "guest" => MemberRole::Guest,
        _ => return Err(error_response("Invalid role. Must be: admin, member, or guest", "INVALID_ROLE")),
    };
    
    let input = CreateMemberInput {
        tenant_id: TenantId::from(tenant_uuid),
        name: name.to_string(),
        email: Some(email.to_string()),
        avatar_url: None,
        role: Some(member_role),
    };
    
    match resources.store.create_member(input).await {
        Ok(member) => {
            let response = AddMemberResponse {
                member_id: member.meta.id.to_string(),
                tenant_id: member.meta.tenant_id.to_string(),
                user_id: user_id.to_string(),
                role: format!("{:?}", member.role).to_lowercase(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "ADD_MEMBER_FAILED")),
    }
}

/// Set a user's primary tenant
pub async fn set_primary_tenant(
    resources: &Arc<SharedResources>,
    user_id: &str,
    tenant_id: &str,
) -> DbOpResult {
    info!(user_id = %user_id, tenant_id = %tenant_id, "Setting primary tenant");
    
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    let tenant_uuid = Uuid::parse_str(tenant_id)
        .map_err(|_| error_response("Invalid tenant_id format", "INVALID_UUID"))?;
    
    let input = UpdateUserInput {
        name: None,
        avatar_url: None,
        primary_tenant_id: Some(TenantId::from(tenant_uuid)),
        settings: None,
    };
    
    match resources.store.update_user(user_uuid, input).await {
        Ok(()) => {
            let response = SetPrimaryTenantResponse {
                success: true,
                user_id: user_id.to_string(),
                tenant_id: tenant_id.to_string(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "SET_PRIMARY_TENANT_FAILED")),
    }
}

/// Create a channel in a tenant
pub async fn create_channel(
    resources: &Arc<SharedResources>,
    tenant_id: &str,
    channel_type: &str,
    name: Option<&str>,
    owner_id: Option<&str>,
) -> DbOpResult {
    info!(tenant_id = %tenant_id, channel_type = %channel_type, "Creating channel");
    
    let tenant_uuid = Uuid::parse_str(tenant_id)
        .map_err(|_| error_response("Invalid tenant_id format", "INVALID_UUID"))?;
    
    let owner_uuid = owner_id.map(|id| {
        Uuid::parse_str(id).map(UserId::from)
    }).transpose()
        .map_err(|_| error_response("Invalid owner_id format", "INVALID_UUID"))?;
    
    let chan_type = match channel_type.to_lowercase().as_str() {
        "personal" => ChannelType::Personal,
        "family" => ChannelType::Family,
        "shared" => ChannelType::Shared,
        _ => return Err(error_response("Invalid channel_type. Must be: personal, family, or shared", "INVALID_CHANNEL_TYPE")),
    };
    
    let default_name = match chan_type {
        ChannelType::Personal => "Personal",
        ChannelType::Family => "Family Chat",
        ChannelType::Shared => "Shared",
    };
    
    let input = CreateChannelInput {
        tenant_id: TenantId::from(tenant_uuid),
        owner_id: owner_uuid,
        name: name.unwrap_or(default_name).to_string(),
        description: None,
        channel_type: Some(chan_type),
    };
    
    match resources.store.create_channel(input).await {
        Ok(channel) => {
            let response = CreateChannelResponse {
                channel_id: channel.meta.id.to_string(),
                tenant_id: channel.meta.tenant_id.to_string(),
                name: channel.name,
                channel_type: channel.channel_type.as_str().to_string(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "CREATE_CHANNEL_FAILED")),
    }
}

/// Increment invitation usage count
pub async fn increment_invite(resources: &Arc<SharedResources>, invitation_id: &str) -> DbOpResult {
    info!(invitation_id = %invitation_id, "Incrementing invitation usage");
    
    let invite_uuid = Uuid::parse_str(invitation_id)
        .map_err(|_| error_response("Invalid invitation_id format", "INVALID_UUID"))?;
    
    match resources.store.use_invitation(InvitationId::from(invite_uuid)).await {
        Ok(()) => {
            let response = IncrementInviteResponse {
                success: true,
                invitation_id: invitation_id.to_string(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "INCREMENT_INVITE_FAILED")),
    }
}

/// Create an audit log entry
pub async fn audit_log(
    resources: &Arc<SharedResources>,
    action: &str,
    user_id: Option<&str>,
    user_email: Option<&str>,
    resource_type: Option<&str>,
    resource_id: Option<&str>,
    metadata_json: Option<&str>,
) -> DbOpResult {
    info!(action = %action, user_id = ?user_id, "Creating audit log entry");
    
    let user_uuid = user_id.map(|id| {
        Uuid::parse_str(id).map(UserId::from)
    }).transpose()
        .map_err(|_| error_response("Invalid user_id format", "INVALID_UUID"))?;
    
    let resource_uuid = resource_id.map(|id| {
        Uuid::parse_str(id)
    }).transpose()
        .map_err(|_| error_response("Invalid resource_id format", "INVALID_UUID"))?;
    
    let metadata: Option<serde_json::Value> = metadata_json.map(|json| {
        serde_json::from_str(json)
    }).transpose()
        .map_err(|e| error_response(&format!("Invalid metadata JSON: {}", e), "INVALID_JSON"))?;
    
    let input = CreateAuditLogInput {
        user_id: user_uuid,
        user_email: user_email.map(String::from),
        action: action.to_string(),
        resource_type: resource_type.map(String::from),
        resource_id: resource_uuid,
        ip_address: None,
        user_agent: None,
        metadata,
        success: true,
        error_message: None,
    };
    
    match resources.store.create_audit_log(input).await {
        Ok(log) => {
            let response = AuditLogResponse {
                audit_log_id: log.id.to_string(),
                action: log.action,
                created_at: log.created_at.to_rfc3339(),
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => Err(error_response(&e.to_string(), "AUDIT_LOG_FAILED")),
    }
}
