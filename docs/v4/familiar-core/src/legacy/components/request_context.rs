//! Request Context Component
//!
//! Captures HTTP request metadata for audit logging and security tracking.
//! Used by types that need to record the origin of an action.

use serde::{Deserialize, Serialize};

/// HTTP request context for audit/logging
/// 
/// Captures metadata about the HTTP request that triggered an action.
/// Used for security auditing, rate limiting, and debugging.
///
/// ## Usage
///
/// Embed in types that need request tracking:
/// ```rust,ignore
/// pub struct AuditLogEntry {
///     // ... other fields
///     #[serde(flatten)]
///     pub request_context: RequestContext,
/// }
/// ```
///
/// ## Fields Replaced
/// 
/// This component replaces the common pattern of:
/// - `pub ip_address: Option<String>`
/// - `pub user_agent: Option<String>`
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RequestContext {
    /// Client IP address (may be from X-Forwarded-For in proxied environments)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    
    /// Client User-Agent header
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self { ip_address, user_agent }
    }

    /// Create from IP only
    pub fn from_ip(ip: impl Into<String>) -> Self {
        Self {
            ip_address: Some(ip.into()),
            user_agent: None,
        }
    }

    /// Create empty context (for system-initiated actions)
    pub fn system() -> Self {
        Self::default()
    }

    /// Check if this is a system context (no client info)
    pub fn is_system(&self) -> bool {
        self.ip_address.is_none() && self.user_agent.is_none()
    }

    /// Get a display-safe version of the IP (for logs)
    pub fn safe_ip(&self) -> &str {
        self.ip_address.as_deref().unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new(
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );
        assert_eq!(ctx.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(ctx.user_agent, Some("Mozilla/5.0".to_string()));
        assert!(!ctx.is_system());
    }

    #[test]
    fn test_request_context_system() {
        let ctx = RequestContext::system();
        assert!(ctx.is_system());
        assert_eq!(ctx.safe_ip(), "unknown");
    }
}




