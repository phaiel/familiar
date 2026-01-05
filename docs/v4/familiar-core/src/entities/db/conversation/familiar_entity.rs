//! SeaORM entity: FamiliarEntity
//! Generated from: database/FamiliarEntityModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{ChannelId, MessageId, EntityId, TenantId, UserId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum FamiliarEntityType {
    #[sea_orm(string_value = "bond")]
    Bond,
    #[sea_orm(string_value = "filament")]
    Filament,
    #[sea_orm(string_value = "focus")]
    Focus,
    #[sea_orm(string_value = "intent")]
    Intent,
    #[sea_orm(string_value = "moment")]
    Moment,
    #[sea_orm(string_value = "motif")]
    Motif,
    #[sea_orm(string_value = "pulse")]
    Pulse,
    #[sea_orm(string_value = "thread")]
    Thread,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum EntityStatus {
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "auto_spawned")]
    AutoSpawned,
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "rejected")]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "familiar_entities")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: EntityId,
    pub content: String,
    pub created_at: DateTimeUtc,
    pub entity_type: FamiliarEntityType,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub physics: Option<Json>,
    #[sea_orm(nullable)]
    pub qdrant_collection: Option<String>,
    #[sea_orm(nullable)]
    pub qdrant_point_id: Option<Uuid>,
    #[sea_orm(nullable)]
    pub reviewed_at: Option<DateTimeUtc>,
    #[sea_orm(nullable)]
    pub reviewed_by: Option<UserId>,
    #[sea_orm(nullable)]
    pub source_channel_id: Option<ChannelId>,
    #[sea_orm(nullable)]
    pub source_message_id: Option<MessageId>,
    pub status: EntityStatus,
    #[sea_orm(nullable)]
    pub subject: Option<String>,
    pub tenant_id: TenantId,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
