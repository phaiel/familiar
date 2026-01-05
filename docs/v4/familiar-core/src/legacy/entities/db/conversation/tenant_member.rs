//! SeaORM entity: TenantMember
//! Generated from: database/TenantMemberModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{TenantId, UserId, ThreadId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum MemberRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "guest")]
    Guest,
    #[sea_orm(string_value = "member")]
    Member,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "tenant_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: ThreadId,
    #[sea_orm(nullable)]
    pub avatar_url: Option<String>,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub email: Option<String>,
    pub name: String,
    pub role: MemberRole,
    #[sea_orm(column_type = "JsonBinary")]
    pub settings: Json,
    pub tenant_id: TenantId,
    pub updated_at: DateTimeUtc,
    pub user_id: UserId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "crate::entities::db::auth::user::Entity",
        from = "Column::UserId",
        to = "crate::entities::db::auth::user::Column::Id"
    )]
    User,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<crate::entities::db::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
