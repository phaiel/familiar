//! SeaORM entity: User
//! Generated from: database/UserModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{TenantId, UserId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: UserId,
    #[sea_orm(nullable)]
    pub avatar_url: Option<String>,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub deletion_requested_at: Option<DateTimeUtc>,
    #[sea_orm(unique)]
    pub email: String,
    pub email_verified: bool,
    #[sea_orm(column_type = "JsonBinary")]
    pub gdpr_consents: Json,
    pub name: String,
    #[sea_orm(nullable)]
    pub password_hash: Option<String>,
    #[sea_orm(nullable)]
    pub primary_tenant_id: Option<TenantId>,
    #[sea_orm(column_type = "JsonBinary")]
    pub settings: Json,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::session::Entity")]
    Sessions,
    #[sea_orm(has_many = "super::consent::Entity")]
    Consents,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl Related<super::consent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Consents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
