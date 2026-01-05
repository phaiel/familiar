//! SeaORM Database Entities
//!
//! This module provides SeaORM entity definitions for all database tables.
//! Entities are organized by domain:
//!
//! - `conversation`: Tenants, channels, messages, and familiar entities
//! - `auth`: Users, sessions, invitations, and audit logs
//! - `physics`: Entity registry, field excitations, quantum states, and content
//! - `task`: Async task tracking for Kafka command processing
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::entities::db::conversation::channel;
//!
//! // Query all channels
//! let channels = channel::Entity::find().all(&db).await?;
//!
//! // Insert a new channel
//! let model = channel::ActiveModel {
//!     id: Set(Uuid::new_v4()),
//!     name: Set("My Channel".to_string()),
//!     ..Default::default()
//! };
//! channel::Entity::insert(model).exec(&db).await?;
//! ```

pub mod conversation;
pub mod auth;
pub mod physics;
pub mod task;
pub mod trace;
pub mod optimistic_lock;

// Re-export common entities for convenience
pub use conversation::{tenant, tenant_member, channel, message, familiar_entity};
pub use auth::{user, session, magic_link, invitation, join_request, consent, audit};
pub use physics::{entity_registry, field_excitation, quantum_state, content};
pub use task::async_task;
pub use trace::course_trace;
