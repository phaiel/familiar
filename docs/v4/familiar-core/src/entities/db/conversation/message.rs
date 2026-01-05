//! SeaORM entity: Message
//! Generated from: database/MessageModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{UserId, MessageId, ChannelId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum MessageRole {
    #[sea_orm(string_value = "assistant")]
    Assistant,
    #[sea_orm(string_value = "system")]
    System,
    #[sea_orm(string_value = "user")]
    User,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: MessageId,
    #[sea_orm(nullable)]
    pub agent_speaker: Option<String>,
    pub channel_id: ChannelId,
    pub content: String,
    pub created_at: DateTimeUtc,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    #[sea_orm(nullable)]
    pub parent_id: Option<MessageId>,
    pub role: MessageRole,
    #[sea_orm(nullable)]
    pub sender_id: Option<UserId>,
    #[sea_orm(column_type = "JsonBinary")]
    pub thinking_steps: Json,
    #[sea_orm(column_type = "JsonBinary")]
    pub tool_calls: Json,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub weave_result: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::channel::Entity",
        from = "Column::ChannelId",
        to = "super::channel::Column::Id"
    )]
    Channel,
}

impl Related<super::channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channel.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
