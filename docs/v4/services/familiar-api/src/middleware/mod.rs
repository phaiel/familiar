//! Middleware for authentication and authorization
//!
//! This module provides Axum middleware for:
//! - JWT token validation
//! - User extraction from tokens
//! - Tenant isolation enforcement

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

// ============================================================================
// Auth Types
// ============================================================================

/// Authenticated user extracted from JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// User's unique ID
    pub user_id: Uuid,
    /// User's tenant (family) ID
    pub tenant_id: Uuid,
    /// User's email (from token)
    pub email: Option<String>,
    /// User's role within the tenant
    pub role: Option<String>,
}

/// JWT claims structure (must match what the auth service issues)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Email (optional)
    pub email: Option<String>,
    /// Role (optional)
    pub role: Option<String>,
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: Option<usize>,
}

/// Auth error response
#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: bool,
    pub message: String,
    pub code: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "TOKEN_EXPIRED" => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

// ============================================================================
// Middleware Functions
// ============================================================================

/// Middleware that requires authentication
/// 
/// Extracts JWT from Authorization header, validates it, and injects
/// AuthenticatedUser into request extensions.
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Get JWT secret from config
    let jwt_secret = state.config.auth.jwt_secret.as_bytes();
    
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    
    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err(AuthError {
                error: true,
                message: "Missing or invalid Authorization header".to_string(),
                code: "UNAUTHORIZED".to_string(),
            });
        }
    };
    
    // Decode and validate JWT
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| {
        let (message, code) = match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                ("Token has expired".to_string(), "TOKEN_EXPIRED".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidToken => {
                ("Invalid token format".to_string(), "UNAUTHORIZED".to_string())
            }
            _ => (format!("Token validation failed: {}", e), "UNAUTHORIZED".to_string()),
        };
        AuthError { error: true, message, code }
    })?;
    
    let claims = token_data.claims;
    
    // Parse UUIDs from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError {
        error: true,
        message: "Invalid user ID in token".to_string(),
        code: "UNAUTHORIZED".to_string(),
    })?;
    
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(|_| AuthError {
        error: true,
        message: "Invalid tenant ID in token".to_string(),
        code: "UNAUTHORIZED".to_string(),
    })?;
    
    // Create authenticated user
    let auth_user = AuthenticatedUser {
        user_id,
        tenant_id,
        email: claims.email,
        role: claims.role,
    };
    
    // Inject into request extensions
    request.extensions_mut().insert(auth_user);
    
    Ok(next.run(request).await)
}

/// Middleware that allows optional authentication
/// 
/// If Authorization header is present, validates it and injects AuthenticatedUser.
/// If not present, continues without authentication (for public endpoints).
pub async fn optional_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let jwt_secret = state.config.auth.jwt_secret.as_bytes();
    
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            if header_str.starts_with("Bearer ") {
                let token = &header_str[7..];
                
                if let Ok(token_data) = decode::<JwtClaims>(
                    token,
                    &DecodingKey::from_secret(jwt_secret),
                    &Validation::new(Algorithm::HS256),
                ) {
                    let claims = token_data.claims;
                    
                    if let (Ok(user_id), Ok(tenant_id)) = (
                        Uuid::parse_str(&claims.sub),
                        Uuid::parse_str(&claims.tenant_id),
                    ) {
                        let auth_user = AuthenticatedUser {
                            user_id,
                            tenant_id,
                            email: claims.email,
                            role: claims.role,
                        };
                        request.extensions_mut().insert(auth_user);
                    }
                }
            }
        }
    }
    
    next.run(request).await
}

/// Development-only middleware that creates a mock user
/// 
/// WARNING: Only use in development mode. Creates a fake authenticated user
/// for testing purposes.
#[cfg(debug_assertions)]
pub async fn dev_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Only use dev auth if dev_mode is enabled
    if state.config.server.dev_mode {
        // Check if already authenticated
        if request.extensions().get::<AuthenticatedUser>().is_none() {
            // Create a dev user
            let dev_user = AuthenticatedUser {
                user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                tenant_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                email: Some("dev@familiar.local".to_string()),
                role: Some("admin".to_string()),
            };
            request.extensions_mut().insert(dev_user);
            tracing::warn!("DEV MODE: Using mock authenticated user");
        }
    }
    
    next.run(request).await
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a JWT token for a user (for testing/auth service)
pub fn generate_token(
    user_id: Uuid,
    tenant_id: Uuid,
    email: Option<String>,
    role: Option<String>,
    secret: &[u8],
    expiry_hours: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = JwtClaims {
        sub: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        email,
        role,
        iat: Some(now),
        exp: now + (expiry_hours as usize * 3600),
    };
    
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_and_decode_token() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let secret = b"test-secret-key-for-jwt-tokens";
        
        let token = generate_token(
            user_id,
            tenant_id,
            Some("test@example.com".to_string()),
            Some("member".to_string()),
            secret,
            24,
        )
        .expect("Token generation should succeed");
        
        let token_data = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret),
            &Validation::new(Algorithm::HS256),
        )
        .expect("Token decoding should succeed");
        
        assert_eq!(token_data.claims.sub, user_id.to_string());
        assert_eq!(token_data.claims.tenant_id, tenant_id.to_string());
        assert_eq!(token_data.claims.email, Some("test@example.com".to_string()));
    }
}







