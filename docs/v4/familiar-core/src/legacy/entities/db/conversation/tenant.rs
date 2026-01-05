//! SeaORM entity: Tenant
//! Generated from: database/TenantModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{TenantId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: TenantId,
    pub created_at: DateTimeUtc,
    pub name: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub settings: Json,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tenant_member::Entity")]
    Members,
    #[sea_orm(has_many = "super::channel::Entity")]
    Channels,
}

impl Related<super::tenant_member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl Related<super::channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
