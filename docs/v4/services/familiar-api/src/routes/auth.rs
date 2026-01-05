//! Authentication API Routes
//!
//! Handles user authentication flows:
//! - Email + password signup/login
//! - Magic link (passwordless) auth
//! - Session management
//! - Current user info

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use tracing::{info, warn};

// Password hashing with Argon2id (from familiar-core)
use familiar_core::primitives::PasswordHash as SecurePasswordHash;

use familiar_core::types::{
    CreateUserInput,
    CreateSessionInput,
    MagicLinkPurpose, CreateMagicLinkInput,
    RecordConsentInput, ConsentType,
    CreateAuditLogInput,
    SignupRequest, LoginRequest, MagicLinkRequest, AuthResponse, CurrentUser,
    SystemEntityMeta,
};
use familiar_core::types::kafka::{EnvelopeV1, Payload, SignupConsents, RequestContext, TenantId, UserId};
use familiar_core::components::Timestamps;
use familiar_core::infrastructure::store::TigerDataStore;

use crate::state::AppState;
use super::{ErrorResponse, SuccessResponse};

// ============================================================================
// Constants
// ============================================================================

const SESSION_COOKIE_NAME: &str = "familiar_session";
const SESSION_DURATION_HOURS: i64 = 24 * 7; // 1 week

// ============================================================================
// Audit Logging
// ============================================================================

/// Log an audit event with proper error handling
///
/// This logs a warning if the audit log fails but does NOT block the operation.
/// Audit logs are important for compliance but should not break user flows.
async fn log_audit_event(store: &TigerDataStore, input: CreateAuditLogInput) {
    if let Err(e) = store.create_audit_log(input.clone()).await {
        warn!(
            action = %input.action,
            user_id = ?input.user_id,
            error = %e,
            "Failed to create audit log (operation succeeded but audit not recorded)"
        );
        // In production, you might want to:
        // 1. Push to a dead-letter queue for retry
        // 2. Emit a metric for monitoring
        // 3. Alert if failure rate exceeds threshold
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Hash a password using Argon2id with per-password random salt
/// 
/// Delegates to familiar_core::primitives::PasswordHash which provides:
/// - Per-password unique salt (prevents rainbow tables)
/// - Memory-hard computation (prevents GPU/ASIC attacks)
/// - Configurable time cost (prevents brute force)
fn hash_password(password: &str) -> Result<String, &'static str> {
    SecurePasswordHash::hash(password)
        .map(|h| h.to_string_for_storage())
        .map_err(|_| "Password hashing failed")
}

/// Verify a password against an Argon2 hash
/// 
/// The hash contains the algorithm, parameters, salt, and hash value,
/// so verification is self-describing.
fn verify_password(password: &str, hash: &str) -> bool {
    let secure_hash = SecurePasswordHash::from_hash(hash);
    secure_hash.verify(password)
}

/// Generate a secure random token
fn generate_token() -> String {
    Uuid::new_v4().to_string() + &Uuid::new_v4().to_string()
}

/// Hash a token for storage
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract session token from headers
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    // Try cookie
    if let Some(cookie) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if part.starts_with(&format!("{}=", SESSION_COOKIE_NAME)) {
                    return Some(part[SESSION_COOKIE_NAME.len() + 1..].to_string());
                }
            }
        }
    }
    
    None
}

/// Extract IP address from headers
fn extract_ip(headers: &HeaderMap) -> Option<String> {
    headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}

/// Extract user agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers.get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct MagicLinkSentResponse {
    pub success: bool,
    pub message: String,
    /// In development, include the token for testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_token: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Response for async task submission
#[derive(Debug, Serialize, ToSchema)]
pub struct TaskSubmittedResponse {
    /// Task ID for polling status
    pub task_id: Uuid,
    /// URL to poll for task status
    pub poll_url: String,
    /// Human-readable message
    pub message: String,
}

/// POST /api/auth/signup - Create new account with email + password
/// 
/// Routes through Kafka/Redpanda for async processing via EnvelopeV1 pattern.
/// Falls back to Windmill if Kafka is unavailable.
pub async fn signup_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    info!(email = %req.email, "Signup attempt");
    tracing::debug!("Signup request received: email={}, name={}", req.email, req.name);

    // Validate consent first (client-side validation)
    if !req.accept_terms || !req.accept_privacy {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "You must accept the terms of service and privacy policy".to_string(),
            code: "CONSENT_REQUIRED".to_string(),
        })).into_response();
    }

    // === NEW: Try EnvelopeProducer first (async path) ===
    if let Some(producer) = &state.envelope_producer {
        let task_id = Uuid::new_v4();
        let placeholder_tenant_id = TenantId::new(); // Will be assigned by worker
        let placeholder_user_id = UserId::new(); // Will be assigned by worker
        
        let envelope = EnvelopeV1::command(
            placeholder_tenant_id,
            placeholder_user_id,
            task_id.to_string(), // correlation_id = task_id for polling
            Payload::Signup {
                email: req.email.clone(),
                password: req.password.clone(),
                name: req.name.clone(),
                consents: SignupConsents {
                    terms: req.accept_terms,
                    privacy: req.accept_privacy,
                },
                request_context: Some(RequestContext {
                    ip_address: extract_ip(&headers),
                    user_agent: extract_user_agent(&headers),
                    request_id: Some(task_id.to_string()),
                }),
                invite_code: None,
            },
        );
        
        match producer.send_command(&envelope).await {
            Ok(()) => {
                info!(task_id = %task_id, "Signup command sent via Kafka");
                
                // Return task ID for async polling
                return Json(TaskSubmittedResponse {
                    task_id,
                    poll_url: format!("/api/tasks/{}", task_id),
                    message: "Signup request submitted. Poll the provided URL for status.".to_string(),
                }).into_response();
            }
            Err(e) => {
                tracing::error!("Kafka send failed: {}", e);
                return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                    error: "Signup service temporarily unavailable. Please try again.".to_string(),
                    code: "SERVICE_UNAVAILABLE".to_string(),
                })).into_response();
            }
        }
    }

    // No envelope producer configured
    (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
        error: "Signup service not configured".to_string(),
        code: "SERVICE_NOT_CONFIGURED".to_string(),
    })).into_response()
}

/// POST /api/auth/login - Login with email + password
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    info!(email = %req.email, "Login attempt");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get user
    let user = match store.get_user_by_email(&req.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid email or password".to_string(),
                code: "INVALID_CREDENTIALS".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    // Verify password
    let password_hash = match store.get_user_password_hash(user.meta.id.into()).await {
        Ok(Some(h)) => h,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Password login not enabled for this account".to_string(),
                code: "NO_PASSWORD".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    if !verify_password(&req.password, &password_hash) {
        return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
            error: "Invalid email or password".to_string(),
            code: "INVALID_CREDENTIALS".to_string(),
        })).into_response();
    }

    // Create session
    let ip = extract_ip(&headers);
    let ua = extract_user_agent(&headers);
    let token = generate_token();
    let token_hash = hash_token(&token);
    
    let session = match store.create_session(CreateSessionInput {
        user_id: user.meta.id,
        token_hash,
        user_agent: ua.clone(),
        ip_address: ip.clone(),
        expires_in_hours: SESSION_DURATION_HOURS,
    }).await {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "CREATE_SESSION_FAILED".to_string(),
            })).into_response();
        }
    };

    // Audit log (non-blocking - logs warning on failure)
    log_audit_event(store, CreateAuditLogInput {
        user_id: Some(user.meta.id),
        user_email: Some(user.email.clone()),
        action: "login".to_string(),
        resource_type: Some("user".to_string()),
        resource_id: Some(user.meta.id.into()),
        ip_address: ip,
        user_agent: ua,
        metadata: None,
        success: true,
        error_message: None,
    }).await;

    // Check if user needs family
    let memberships = store.get_user_memberships(user.meta.id.into()).await.unwrap_or_default();
    let needs_family = memberships.is_empty();

    // Build response
    let response = AuthResponse {
        user,
        session: familiar_core::types::SessionCreated {
            session_id: session.id,
            token,
            expires_at: session.expires_at,
        },
        is_new_user: false,
        needs_family,
    };

    Json(response).into_response()
}

/// POST /api/auth/magic-link - Request a magic link
pub async fn magic_link_request_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MagicLinkRequest>,
) -> impl IntoResponse {
    info!(email = %req.email, "Magic link request");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Check if user exists to determine purpose
    let user_exists = store.get_user_by_email(&req.email).await.ok().flatten().is_some();
    let purpose = if user_exists { MagicLinkPurpose::Login } else { MagicLinkPurpose::Signup };

    // Generate token
    let token = generate_token();
    let token_hash = hash_token(&token);

    // Create magic link
    let metadata = req.invite_code.map(|code| serde_json::json!({ "invite_code": code }));
    
    match store.create_magic_link(CreateMagicLinkInput {
        email: req.email.clone(),
        purpose,
        metadata,
        expires_in_minutes: 15,
    }, token_hash).await {
        Ok(_) => {
            // In production, send email here
            // For development, include token in response
            let is_dev = state.config.server.dev_mode;
            
            Json(MagicLinkSentResponse {
                success: true,
                message: "Magic link sent to your email".to_string(),
                dev_token: if is_dev { Some(token) } else { None },
            }).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "CREATE_MAGIC_LINK_FAILED".to_string(),
            })).into_response()
        }
    }
}

/// GET /api/auth/magic-link/:token - Consume a magic link
pub async fn magic_link_consume_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> impl IntoResponse {
    info!("Magic link consume attempt");

    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    let token_hash = hash_token(&token);

    // Get magic link
    let magic_link = match store.get_magic_link_by_token(&token_hash).await {
        Ok(Some(ml)) => ml,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: "Invalid or expired magic link".to_string(),
                code: "INVALID_MAGIC_LINK".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    // Check if already used
    if magic_link.used_at.is_some() {
        return (StatusCode::GONE, Json(ErrorResponse {
            error: "This magic link has already been used".to_string(),
            code: "MAGIC_LINK_USED".to_string(),
        })).into_response();
    }

    // Check if expired
    if magic_link.expires_at < chrono::Utc::now() {
        return (StatusCode::GONE, Json(ErrorResponse {
            error: "This magic link has expired".to_string(),
            code: "MAGIC_LINK_EXPIRED".to_string(),
        })).into_response();
    }

    // Mark as used
    let _ = store.consume_magic_link(magic_link.id).await;

    // Get or create user
    let (user, is_new) = match store.get_user_by_email(&magic_link.email).await {
        Ok(Some(u)) => (u, false),
        Ok(None) => {
            // Create new user
            let name = magic_link.email.split('@').next().unwrap_or("User").to_string();
            match store.create_user(CreateUserInput {
                email: magic_link.email.clone(),
                name,
                password_hash: None,
                avatar_url: None,
                primary_tenant_id: None,
            }).await {
                Ok(u) => {
                    // Verify email immediately for magic link signups
                    let _ = store.verify_user_email(u.meta.id.into()).await;
                    (u, true)
                }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                        error: e.to_string(),
                        code: "CREATE_USER_FAILED".to_string(),
                    })).into_response();
                }
            }
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    // Create session
    let ip = extract_ip(&headers);
    let ua = extract_user_agent(&headers);
    let session_token = generate_token();
    let session_token_hash = hash_token(&session_token);
    
    let session = match store.create_session(CreateSessionInput {
        user_id: user.meta.id,
        token_hash: session_token_hash,
        user_agent: ua.clone(),
        ip_address: ip.clone(),
        expires_in_hours: SESSION_DURATION_HOURS,
    }).await {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "CREATE_SESSION_FAILED".to_string(),
            })).into_response();
        }
    };

    // Audit log (non-blocking - logs warning on failure)
    log_audit_event(store, CreateAuditLogInput {
        user_id: Some(user.meta.id),
        user_email: Some(user.email.clone()),
        action: if is_new { "signup_magic_link" } else { "login_magic_link" }.to_string(),
        resource_type: Some("user".to_string()),
        resource_id: Some(user.meta.id.into()),
        ip_address: ip,
        user_agent: ua,
        metadata: None,
        success: true,
        error_message: None,
    }).await;

    // Refresh user to get verified status
    let user = store.get_user(user.meta.id.into()).await.ok().flatten().unwrap_or(user);

    // Check if user needs family
    let memberships = store.get_user_memberships(user.meta.id.into()).await.unwrap_or_default();
    let needs_family = memberships.is_empty();

    // Build response
    let response = AuthResponse {
        user,
        session: familiar_core::types::SessionCreated {
            session_id: session.id,
            token: session_token,
            expires_at: session.expires_at,
        },
        is_new_user: is_new,
        needs_family,
    };

    Json(response).into_response()
}

/// POST /api/auth/logout - End current session
pub async fn logout_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return Json(SuccessResponse {
                success: true,
                message: Some("Logged out".to_string()),
            }).into_response();
        }
    };

    // Get token
    if let Some(token) = extract_session_token(&headers) {
        let token_hash = hash_token(&token);
        
        // Get user for audit log
        if let Ok(Some(user_id)) = store.validate_session(&token_hash).await {
            if let Ok(Some(user)) = store.get_user(user_id).await {
                // Delete all sessions for user
                let _ = store.delete_user_sessions(user_id).await;
                
                // Audit log (non-blocking - logs warning on failure)
                log_audit_event(store, CreateAuditLogInput {
                    user_id: Some(user_id.into()),
                    user_email: Some(user.email),
                    action: "logout".to_string(),
                    resource_type: None,
                    resource_id: None,
                    ip_address: extract_ip(&headers),
                    user_agent: extract_user_agent(&headers),
                    metadata: None,
                    success: true,
                    error_message: None,
                }).await;
            }
        }
    }

    Json(SuccessResponse {
        success: true,
        message: Some("Logged out successfully".to_string()),
    }).into_response()
}

/// GET /api/auth/me - Get current authenticated user
pub async fn me_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let store = match &state.store {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
                error: "Database not configured".to_string(),
                code: "DB_NOT_CONFIGURED".to_string(),
            })).into_response();
        }
    };

    // Get token
    let token = match extract_session_token(&headers) {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "No session token provided".to_string(),
                code: "NO_TOKEN".to_string(),
            })).into_response();
        }
    };

    let token_hash = hash_token(&token);

    // Validate session
    let user_id = match store.validate_session(&token_hash).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid or expired session".to_string(),
                code: "INVALID_SESSION".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    // Get user
    let user = match store.get_user(user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: e.to_string(),
                code: "DB_ERROR".to_string(),
            })).into_response();
        }
    };

    // Get memberships
    let memberships = store.get_user_memberships(user_id).await.unwrap_or_default();

    Json(CurrentUser {
        user,
        memberships,
    }).into_response()
}
