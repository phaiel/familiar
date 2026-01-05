//! Conversation domain data access (Tenants, Channels, Messages)

use sea_orm::{prelude::*, ActiveValue::Set, QueryOrder, QuerySelect, sea_query::Expr};
use uuid::Uuid;
use chrono::Utc;

use crate::infrastructure::store::TigerDataStore;
use crate::internal::DbStoreError;
use crate::primitives::{UserId, MessageId, ChannelId, TenantId, EntityId, ThreadId};
use crate::types::conversation::{
    Tenant, CreateTenantInput, TenantMember, CreateMemberInput, MemberRole,
    Channel, CreateChannelInput, ChannelType, ListChannelsOptions,
    Message, CreateMessageInput, MessageRole, ConversationMessage, ListMessagesOptions,
    FamiliarEntity, CreateEntityInput, UpdateEntityStatusInput,
    FamiliarEntityType, EntityStatus, ListEntitiesOptions,
};
use crate::types::base::{EntityMeta, SystemEntityMeta};
use crate::components::Timestamps;
use crate::entities::db::conversation::{tenant, tenant_member, channel, message, familiar_entity};

impl TigerDataStore {
    // ========================================================================
    // Tenant Operations
    // ========================================================================

    pub async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, DbStoreError> {
        let now = Utc::now();
        let id = TenantId::new();
        
        let model = tenant::ActiveModel {
            id: Set(id),
            name: Set(input.name.clone()),
            settings: Set(input.settings.clone().unwrap_or_default()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        tenant::Entity::insert(model).exec(&self.db).await?;
        
        Ok(Tenant {
            meta: SystemEntityMeta {
                id: id.as_uuid().into(),
                timestamps: Timestamps { created_at: now, updated_at: now },
            },
            name: input.name,
            settings: input.settings.unwrap_or_default(),
        })
    }

    pub async fn get_tenant(&self, tenant_id: TenantId) -> Result<Option<Tenant>, DbStoreError> {
        let result = tenant::Entity::find_by_id(tenant_id).one(&self.db).await?;
        
        Ok(result.map(|m| Tenant {
            meta: SystemEntityMeta {
                id: m.id.as_uuid().into(),
                timestamps: Timestamps { created_at: m.created_at, updated_at: m.updated_at },
            },
            name: m.name,
            settings: m.settings,
        }))
    }

    // ========================================================================
    // Member Operations
    // ========================================================================

    pub async fn create_member(&self, input: CreateMemberInput) -> Result<TenantMember, DbStoreError> {
        let now = Utc::now();
        let thread_id = ThreadId::new();
        
        let model = tenant_member::ActiveModel {
            id: Set(thread_id),
            user_id: Set(input.user_id),
            tenant_id: Set(input.tenant_id),
            name: Set(input.name.clone()),
            email: Set(input.email.clone()),
            avatar_url: Set(input.avatar_url.clone()),
            role: Set(match input.role.unwrap_or(MemberRole::Member) {
                MemberRole::Admin => tenant_member::MemberRole::Admin,
                MemberRole::Member => tenant_member::MemberRole::Member,
                MemberRole::Guest => tenant_member::MemberRole::Guest,
            }),
            settings: Set(serde_json::json!({})),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        tenant_member::Entity::insert(model).exec(&self.db).await?;
        
        Ok(TenantMember {
            meta: EntityMeta {
                id: thread_id.as_uuid().into(),
                tenant_id: input.tenant_id,
                timestamps: Timestamps { created_at: now, updated_at: now },
            },
            name: input.name,
            email: input.email,
            avatar_url: input.avatar_url,
            role: input.role.unwrap_or(MemberRole::Member),
            settings: serde_json::json!({}),
        })
    }

    pub async fn get_tenant_members(&self, tenant_id: TenantId) -> Result<Vec<TenantMember>, DbStoreError> {
        let results = tenant_member::Entity::find()
            .filter(tenant_member::Column::TenantId.eq(tenant_id))
            .order_by_asc(tenant_member::Column::Name)
            .all(&self.db)
            .await?;
        
        Ok(results.into_iter().map(|m| TenantMember {
            meta: EntityMeta {
                id: m.id.as_uuid().into(),
                tenant_id: m.tenant_id,
                timestamps: Timestamps { created_at: m.created_at, updated_at: m.updated_at },
            },
            name: m.name,
            email: m.email,
            avatar_url: m.avatar_url,
            role: match m.role {
                tenant_member::MemberRole::Admin => MemberRole::Admin,
                tenant_member::MemberRole::Member => MemberRole::Member,
                tenant_member::MemberRole::Guest => MemberRole::Guest,
            },
            settings: m.settings,
        }).collect())
    }

    // ========================================================================
    // Channel Operations
    // ========================================================================

    pub async fn create_channel(&self, input: CreateChannelInput) -> Result<Channel, DbStoreError> {
        let now = Utc::now();
        let id = ChannelId::new();
        
        let model = channel::ActiveModel {
            id: Set(id),
            tenant_id: Set(input.tenant_id),
            owner_id: Set(input.owner_id),
            name: Set(input.name.clone()),
            description: Set(input.description.clone()),
            channel_type: Set(match input.channel_type.unwrap_or_default() {
                ChannelType::Personal => channel::ChannelType::Personal,
                ChannelType::Family => channel::ChannelType::Family,
                ChannelType::Shared => channel::ChannelType::Shared,
            }),
            settings: Set(serde_json::json!({})),
            created_at: Set(now),
            updated_at: Set(now),
            version: Set(0), // Initial version for optimistic locking
        };
        
        channel::Entity::insert(model).exec(&self.db).await?;
        
        Ok(Channel {
            meta: EntityMeta {
                id: id.into(),
                tenant_id: input.tenant_id,
                timestamps: Timestamps { created_at: now, updated_at: now },
            },
            owner_id: input.owner_id,
            name: input.name,
            description: input.description,
            channel_type: input.channel_type.unwrap_or_default(),
            settings: serde_json::json!({}),
        })
    }

    pub async fn get_channels(&self, tenant_id: Uuid, options: ListChannelsOptions) -> Result<Vec<Channel>, DbStoreError> {
        let mut query = channel::Entity::find().filter(channel::Column::TenantId.eq(tenant_id));
        
        if let Some(ct) = options.channel_type {
            query = query.filter(channel::Column::ChannelType.eq(match ct {
                ChannelType::Personal => channel::ChannelType::Personal,
                ChannelType::Family => channel::ChannelType::Family,
                ChannelType::Shared => channel::ChannelType::Shared,
            }));
        }
        
        if let Some(owner) = options.owner_id {
            query = query.filter(channel::Column::OwnerId.eq(owner.as_uuid()));
        }
        
        let results = query.order_by_desc(channel::Column::CreatedAt).all(&self.db).await?;
        
        Ok(results.into_iter().map(|m| Channel {
            meta: EntityMeta {
                id: m.id.into(),
                tenant_id: m.tenant_id.into(),
                timestamps: Timestamps { created_at: m.created_at, updated_at: m.updated_at },
            },
            owner_id: m.owner_id.map(|o| o.into()),
            name: m.name,
            description: m.description,
            channel_type: match m.channel_type {
                channel::ChannelType::Personal => ChannelType::Personal,
                channel::ChannelType::Family => ChannelType::Family,
                channel::ChannelType::Shared => ChannelType::Shared,
            },
            settings: m.settings,
        }).collect())
    }

    pub async fn get_channel(&self, channel_id: Uuid) -> Result<Option<Channel>, DbStoreError> {
        let result = channel::Entity::find_by_id(channel_id).one(&self.db).await?;
        
        Ok(result.map(|m| Channel {
            meta: EntityMeta {
                id: m.id.into(),
                tenant_id: m.tenant_id.into(),
                timestamps: Timestamps { created_at: m.created_at, updated_at: m.updated_at },
            },
            owner_id: m.owner_id.map(|o| o.into()),
            name: m.name,
            description: m.description,
            channel_type: match m.channel_type {
                channel::ChannelType::Personal => ChannelType::Personal,
                channel::ChannelType::Family => ChannelType::Family,
                channel::ChannelType::Shared => ChannelType::Shared,
            },
            settings: m.settings,
        }))
    }

    // ========================================================================
    // Message Operations
    // ========================================================================

    pub async fn create_message(&self, input: CreateMessageInput) -> Result<Message, DbStoreError> {
        let now = Utc::now();
        let id = MessageId::new();
        
        let model = message::ActiveModel {
            id: Set(id),
            channel_id: Set(input.channel_id),
            sender_id: Set(input.sender_id),
            parent_id: Set(input.parent_id),
            role: Set(match input.role {
                MessageRole::User => message::MessageRole::User,
                MessageRole::Assistant => message::MessageRole::Assistant,
                MessageRole::System => message::MessageRole::System,
            }),
            content: Set(input.content.clone()),
            agent_speaker: Set(input.agent_speaker.clone()),
            thinking_steps: Set(input.thinking_steps.clone().unwrap_or_default()),
            tool_calls: Set(input.tool_calls.clone().unwrap_or_default()),
            weave_result: Set(input.weave_result.clone()),
            metadata: Set(input.metadata.clone().unwrap_or_default()),
            created_at: Set(now),
        };
        
        message::Entity::insert(model).exec(&self.db).await?;
        
        Ok(Message {
            id: id.as_uuid().into(),
            channel_id: input.channel_id,
            sender_id: input.sender_id,
            parent_id: input.parent_id,
            role: input.role,
            content: input.content,
            agent_speaker: input.agent_speaker,
            thinking_steps: input.thinking_steps.unwrap_or_default(),
            tool_calls: input.tool_calls.unwrap_or_default(),
            weave_result: input.weave_result,
            metadata: input.metadata.unwrap_or_default(),
            created_at: now,
        })
    }

    pub async fn get_messages(&self, channel_id: ChannelId, options: ListMessagesOptions) -> Result<Vec<Message>, DbStoreError> {
        let results: Vec<message::Model> = message::Entity::find()
            .filter(message::Column::ChannelId.eq(channel_id))
            .order_by_desc(message::Column::CreatedAt)
            .limit(options.limit.unwrap_or(50) as u64)
            .all(&self.db)
            .await?;
        
        Ok(results.into_iter().map(|m| Message {
            id: m.id.as_uuid().into(),
            channel_id: m.channel_id,
            sender_id: m.sender_id,
            parent_id: m.parent_id,
            role: match m.role {
                message::MessageRole::User => MessageRole::User,
                message::MessageRole::Assistant => MessageRole::Assistant,
                message::MessageRole::System => MessageRole::System,
            },
            content: m.content,
            agent_speaker: m.agent_speaker,
            thinking_steps: m.thinking_steps,
            tool_calls: m.tool_calls,
            weave_result: m.weave_result,
            metadata: m.metadata,
            created_at: m.created_at,
        }).collect())
    }

    pub async fn get_conversation_history(&self, channel_id: ChannelId, limit: i64) -> Result<Vec<ConversationMessage>, DbStoreError> {
        let results: Vec<message::Model> = message::Entity::find()
            .filter(message::Column::ChannelId.eq(channel_id))
            .order_by_desc(message::Column::CreatedAt)
            .limit(limit as u64)
            .all(&self.db)
            .await?;
        
        Ok(results.into_iter().map(|m| ConversationMessage {
            role: match m.role {
                message::MessageRole::User => "user".to_string(),
                message::MessageRole::Assistant => "assistant".to_string(),
                message::MessageRole::System => "system".to_string(),
            },
            content: m.content,
        }).collect())
    }

    // ========================================================================
    // FamiliarEntity Operations
    // ========================================================================

    pub async fn create_familiar_entity(&self, input: CreateEntityInput) -> Result<FamiliarEntity, DbStoreError> {
        let now = Utc::now();
        let id = EntityId::new();
        
        let model = familiar_entity::ActiveModel {
            id: Set(id),
            tenant_id: Set(input.tenant_id),
            entity_type: Set(match input.entity_type {
                FamiliarEntityType::Moment => familiar_entity::FamiliarEntityType::Moment,
                FamiliarEntityType::Pulse => familiar_entity::FamiliarEntityType::Pulse,
                FamiliarEntityType::Intent => familiar_entity::FamiliarEntityType::Intent,
                FamiliarEntityType::Thread => familiar_entity::FamiliarEntityType::Thread,
                FamiliarEntityType::Bond => familiar_entity::FamiliarEntityType::Bond,
                FamiliarEntityType::Motif => familiar_entity::FamiliarEntityType::Motif,
                FamiliarEntityType::Filament => familiar_entity::FamiliarEntityType::Filament,
                FamiliarEntityType::Focus => familiar_entity::FamiliarEntityType::Focus,
            }),
            content: Set(input.content.clone()),
            subject: Set(input.subject.clone()),
            physics: Set(input.physics.as_ref().map(|p| serde_json::to_value(p).unwrap_or_default())),
            qdrant_point_id: Set(None),
            qdrant_collection: Set(Some("familiar_entities".to_string())),
            source_message_id: Set(input.source_message_id),
            source_channel_id: Set(input.source_channel_id),
            status: Set(familiar_entity::EntityStatus::Pending),
            reviewed_by: Set(None),
            reviewed_at: Set(None),
            metadata: Set(input.metadata.clone().unwrap_or_default()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        familiar_entity::Entity::insert(model).exec(&self.db).await?;
        
        Ok(FamiliarEntity {
            id: id.into(),
            tenant_id: input.tenant_id,
            entity_type: input.entity_type,
            content: input.content,
            subject: input.subject,
            physics: input.physics,
            qdrant_point_id: None,
            qdrant_collection: Some("familiar_entities".to_string()),
            source_message_id: input.source_message_id,
            source_channel_id: input.source_channel_id,
            status: EntityStatus::Pending,
            reviewed_by: None,
            reviewed_at: None,
            metadata: input.metadata.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_familiar_entities(&self, tenant_id: Uuid, options: ListEntitiesOptions) -> Result<Vec<FamiliarEntity>, DbStoreError> {
        let mut query = familiar_entity::Entity::find().filter(familiar_entity::Column::TenantId.eq(tenant_id));
        
        if let Some(status) = options.status {
            query = query.filter(familiar_entity::Column::Status.eq(match status {
                EntityStatus::Pending => familiar_entity::EntityStatus::Pending,
                EntityStatus::Approved => familiar_entity::EntityStatus::Approved,
                EntityStatus::Rejected => familiar_entity::EntityStatus::Rejected,
                EntityStatus::AutoSpawned => familiar_entity::EntityStatus::AutoSpawned,
            }));
        }
        
        let results = query.order_by_desc(familiar_entity::Column::CreatedAt).all(&self.db).await?;
        
        Ok(results.into_iter().map(|m| FamiliarEntity {
            id: m.id.into(),
            tenant_id: m.tenant_id.into(),
            entity_type: match m.entity_type {
                familiar_entity::FamiliarEntityType::Moment => FamiliarEntityType::Moment,
                familiar_entity::FamiliarEntityType::Pulse => FamiliarEntityType::Pulse,
                familiar_entity::FamiliarEntityType::Intent => FamiliarEntityType::Intent,
                familiar_entity::FamiliarEntityType::Thread => FamiliarEntityType::Thread,
                familiar_entity::FamiliarEntityType::Bond => FamiliarEntityType::Bond,
                familiar_entity::FamiliarEntityType::Motif => FamiliarEntityType::Motif,
                familiar_entity::FamiliarEntityType::Filament => FamiliarEntityType::Filament,
                familiar_entity::FamiliarEntityType::Focus => FamiliarEntityType::Focus,
            },
            content: m.content,
            subject: m.subject,
            physics: m.physics.map(|p| serde_json::from_value(p).unwrap_or_default()),
            qdrant_point_id: m.qdrant_point_id.map(|q| q.into()),
            qdrant_collection: m.qdrant_collection,
            source_message_id: m.source_message_id.map(|s| s.into()),
            source_channel_id: m.source_channel_id.map(|s| s.into()),
            status: match m.status {
                familiar_entity::EntityStatus::Pending => EntityStatus::Pending,
                familiar_entity::EntityStatus::Approved => EntityStatus::Approved,
                familiar_entity::EntityStatus::Rejected => EntityStatus::Rejected,
                familiar_entity::EntityStatus::AutoSpawned => EntityStatus::AutoSpawned,
            },
            reviewed_by: m.reviewed_by.map(|r| r.into()),
            reviewed_at: m.reviewed_at.map(|r| r.into()),
            metadata: m.metadata,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }).collect())
    }

    pub async fn update_entity_status(&self, entity_id: Uuid, input: UpdateEntityStatusInput) -> Result<(), DbStoreError> {
        let now = Utc::now();
        
        familiar_entity::Entity::update_many()
            .col_expr(familiar_entity::Column::Status, Expr::value(match input.status {
                EntityStatus::Pending => "pending",
                EntityStatus::Approved => "approved",
                EntityStatus::Rejected => "rejected",
                EntityStatus::AutoSpawned => "auto_spawned",
            }))
            .col_expr(familiar_entity::Column::ReviewedBy, Expr::value(input.reviewed_by.map(|r| r.as_uuid())))
            .col_expr(familiar_entity::Column::ReviewedAt, Expr::value(Some(now)))
            .col_expr(familiar_entity::Column::UpdatedAt, Expr::value(now))
            .filter(familiar_entity::Column::Id.eq(entity_id))
            .exec(&self.db)
            .await?;
        
        Ok(())
    }

    pub async fn set_entity_qdrant_id(&self, entity_id: Uuid, qdrant_point_id: Uuid, collection: &str) -> Result<(), DbStoreError> {
        let now = Utc::now();
        
        familiar_entity::Entity::update_many()
            .col_expr(familiar_entity::Column::QdrantPointId, Expr::value(Some(qdrant_point_id)))
            .col_expr(familiar_entity::Column::QdrantCollection, Expr::value(Some(collection)))
            .col_expr(familiar_entity::Column::UpdatedAt, Expr::value(now))
            .filter(familiar_entity::Column::Id.eq(entity_id))
            .exec(&self.db)
            .await?;
        
        Ok(())
    }
}

