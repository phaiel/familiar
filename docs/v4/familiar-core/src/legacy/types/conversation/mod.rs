//! Conversation Types
//!
//! Discrete type modules for conversation persistence:
//! - `channel` - Channels (personal + family)
//! - `message` - Messages (conversation history)
//! - `entity` - Familiar entities (spawned from weave_result)
//!
//! Note: Tenant types are at the top level (types/tenant.rs) since
//! they are a core domain concept used across the system.

pub mod channel;
pub mod message;
pub mod entity;

// Re-exports for backwards compatibility
pub use self::channel::{Channel, CreateChannelInput, ChannelType, ListChannelsOptions};
pub use self::message::{Message, CreateMessageInput, ConversationMessage, ListMessagesOptions};
pub use super::MessageRole; // Re-export from canonical location
pub use self::entity::{
    FamiliarEntity, CreateEntityInput, UpdateEntityStatusInput,
    FamiliarEntityType, EntityStatus, EntityPhysics, ListEntitiesOptions,
};

// Re-export tenant types from parent module for backwards compatibility
pub use super::tenant::{Tenant, CreateTenantInput};
pub use super::member::{TenantMember, CreateMemberInput, MemberRole};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_serialization() {
        let ct = ChannelType::Family;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"family\"");
    }

    #[test]
    fn test_entity_type_serialization() {
        let et = FamiliarEntityType::Moment;
        let json = serde_json::to_string(&et).unwrap();
        assert_eq!(json, "\"MOMENT\"");
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"assistant\"");
    }
}
