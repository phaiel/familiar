//! SeaORM entity: Channel
//! Generated from: database/ChannelModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{TenantId, UserId, ChannelId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ChannelType {
    #[sea_orm(string_value = "family")]
    Family,
    #[sea_orm(string_value = "personal")]
    Personal,
    #[sea_orm(string_value = "shared")]
    Shared,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "channels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: ChannelId,
    pub channel_type: ChannelType,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub description: Option<String>,
    pub name: String,
    #[sea_orm(nullable)]
    pub owner_id: Option<UserId>,
    #[sea_orm(column_type = "JsonBinary")]
    pub settings: Json,
    pub tenant_id: TenantId,
    pub updated_at: DateTimeUtc,
    #[sea_orm(default_value = 0)]
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
    #[sea_orm(has_many = "super::message::Entity")]
    Messages,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
