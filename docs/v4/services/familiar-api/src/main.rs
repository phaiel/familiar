//! Familiar API - Central API Gateway
//!
//! This is the main API service for Familiar. All frontends talk to this service.
//! 
//! Architecture:
//! - familiar-api: API gateway (this service) - handles HTTP, persistence, routing
//! - heddle-observer: Worker library - handles LLM classification (no HTTP server)
//! - heddle-ui: Frontend - talks to familiar-api only
//!
//! Endpoints:
//! - POST /api/weave - Process a user message (classification + spawning + persistence)
//! - GET /api/models - List available AI models
//! - GET /api/health - Health check
//!
//! Future endpoints (commented out):
//! - GET /api/entities/moments - List moments
//! - GET /api/entities/pulses - List pulses
//! - etc.

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    middleware as axum_middleware,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod state;
mod routes;
mod clients;
mod openapi;
mod kafka;
mod config;
pub mod middleware;
// windmill is now in clients::windmill (DEPRECATED - use kafka module instead)

use state::AppState;
use config::AppConfig;

#[tokio::main]
async fn main() {
    // Load .env file (for backward compatibility, env vars still work)
    dotenvy::dotenv().ok();

    // Load configuration from config.toml with env var overrides
    // Config loading is now mandatory - no fallbacks to direct env vars
    let app_config = AppConfig::load()
        .expect("Failed to load configuration. Please ensure config.toml exists and environment variables are set correctly.");

    // Extract port before moving app_config
    let port = app_config.server.port;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "familiar_api=debug,tower_http=debug".into())
        )
        .init();

    // Initialize application state
    let state = match AppState::new(app_config).await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::warn!("⚠️ Database connection failed: {}. Running without persistence.", e);
            Arc::new(AppState::without_db())
        }
    };

    // Build router with only currently-used endpoints
    // 
    // Routes are organized into:
    // - Public routes (no auth required): health, login, signup, magic-link
    // - Protected routes (auth required): weave, channels, entities, etc.
    
    // Public routes (no authentication required)
    let public_routes = Router::new()
        // Health
        .route("/api/health", get(routes::health::health_check))
        .route("/health", get(routes::health::health_check))
        // Models (public info)
        .route("/api/models", get(routes::models::get_models))
        .route("/models", get(routes::models::get_models))
        // Auth endpoints (login/signup are public)
        .route("/api/auth/signup", post(routes::auth::signup_handler))
        .route("/api/auth/login", post(routes::auth::login_handler))
        .route("/api/auth/magic-link", post(routes::auth::magic_link_request_handler))
        .route("/api/auth/magic-link/:token", get(routes::auth::magic_link_consume_handler))
        // Invitation lookup by code (public)
        .route("/api/invitations/code/:code", get(routes::invitations::get_invitation_by_code_handler));
    
    // Protected routes (authentication required)
    let protected_routes = Router::new()
        // Heddle endpoints (classification)
        .route("/api/weave", post(routes::weave::weave_handler))
        .route("/weave", post(routes::weave::weave_handler))
        
        // Media endpoints
        .route("/api/media/upload", post(routes::media::upload_handler))
        .route("/api/media/:key", get(routes::media::get_url_handler).delete(routes::media::delete_handler))
        
        // Course WebSocket endpoint (Kafka streaming)
        .route("/api/courses/:id/ws", get(routes::ws::course_ws_handler))
        
        // Agentic endpoints (multi-agent orchestration)
        .route("/api/agentic/command", post(routes::agentic::command_handler))
        // TODO: Implement course-based list/get handlers (threads renamed to courses)
        // .route("/api/agentic/courses", get(routes::agentic::list_courses_handler))
        // .route("/api/agentic/courses/:id", get(routes::agentic::get_course_handler))
        
        // Channel endpoints (multi-tenant conversation persistence)
        .route("/api/tenants/:tenant_id/channels", get(routes::channels::list_channels_handler))
        .route("/api/channels", post(routes::channels::create_channel_handler))
        .route("/api/channels/:id", get(routes::channels::get_channel_handler))
        .route("/api/channels/:id/messages", get(routes::channels::list_messages_handler))
        .route("/api/channels/:id/messages", post(routes::channels::send_message_handler))
        .route("/api/channels/:id/history", get(routes::channels::get_history_handler))
        
        // Entity endpoints (familiar entities - HILT)
        .route("/api/tenants/:tenant_id/entities", get(routes::entities::list_entities_handler))
        .route("/api/entities", post(routes::entities::create_entity_handler))
        .route("/api/entities/:id/status", axum::routing::patch(routes::entities::update_entity_status_handler))
        
        // Auth endpoints (protected - requires login)
        .route("/api/auth/logout", post(routes::auth::logout_handler))
        .route("/api/auth/me", get(routes::auth::me_handler))
        
        // Invitation endpoints (family invitations & join requests)
        .route("/api/invitations", post(routes::invitations::create_invitation_handler))
        .route("/api/invitations/:id/accept", post(routes::invitations::accept_invitation_handler))
        .route("/api/join-requests", post(routes::invitations::create_join_request_handler))
        .route("/api/tenants/:tenant_id/join-requests", get(routes::invitations::list_join_requests_handler))
        .route("/api/join-requests/:id/review", post(routes::invitations::review_join_request_handler))
        
        // Tenant endpoints (create/manage families)
        .route("/api/tenants", post(routes::tenants::create_tenant_handler))
        .route("/api/tenants/:id", get(routes::tenants::get_tenant_handler))
        // Apply auth middleware to protected routes
        .layer(axum_middleware::from_fn_with_state(state.clone(), middleware::require_auth));
    
    // Combine all routes
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // OpenAPI documentation
        .merge(openapi::swagger_ui())
        // CORS for frontend
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("🚀 Familiar API starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect(&format!("Failed to bind to {}", addr));
    
    info!("✅ Listener bound, starting axum server");
    axum::serve(listener, app).await.unwrap();
}
