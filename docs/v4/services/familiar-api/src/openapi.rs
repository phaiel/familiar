//! OpenAPI Documentation
//!
//! Generates OpenAPI spec from API route definitions using utoipa.
//! 
//! The spec is available at `/api/docs` (Swagger UI) and `/api/openapi.json`.
//!
//! ## Adding New Endpoints
//!
//! 1. Add `#[utoipa::path(...)]` to the handler function
//! 2. Add the path to the `paths(...)` list in `ApiDoc`
//! 3. Add any new schema types to `components(schemas(...))`
//!
//! ## Example
//!
//! ```rust,ignore
//! #[utoipa::path(
//!     post,
//!     path = "/api/auth/signup",
//!     request_body = SignupRequest,
//!     responses(
//!         (status = 200, body = AuthResponse),
//!         (status = 400, body = ErrorResponse),
//!     ),
//!     tag = "auth"
//! )]
//! pub async fn signup_handler(...) { ... }
//! ```

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Common error response
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiErrorResponse {
    /// Error message
    pub error: String,
    /// Error code (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// OpenAPI documentation for the Familiar API
/// 
/// NOTE: This is a partial specification. Routes are being migrated to
/// include utoipa annotations incrementally.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Familiar API",
        version = "1.0.0",
        description = r#"
The central API for the Familiar family memory system.

## Authentication

Most endpoints require authentication via Bearer token in the `Authorization` header:

```
Authorization: Bearer <session_token>
```

Obtain a token via `/api/auth/signup` or `/api/auth/login`.

## Multi-tenancy

All data is scoped to a tenant (family). Include the tenant ID in the path or derive it from the authenticated user's primary tenant.
"#,
        license(name = "MIT"),
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development"),
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "tenants", description = "Tenant (family) management"),
        (name = "channels", description = "Channel management"),
        (name = "messages", description = "Message management"),
        (name = "entities", description = "Familiar entity management (HILT)"),
        (name = "agentic", description = "Multi-agent orchestration"),
    ),
    components(
        schemas(
            ApiErrorResponse,
        )
    )
)]
pub struct ApiDoc;

/// Create the Swagger UI service
pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/api/docs")
        .url("/api/openapi.json", ApiDoc::openapi())
}