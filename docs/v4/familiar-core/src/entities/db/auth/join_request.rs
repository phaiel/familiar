//! SeaORM entity: JoinRequest
//! Generated from: database/JoinRequestModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{UserId, TenantId, JoinRequestId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum JoinRequestStatus {
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "rejected")]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "join_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: JoinRequestId,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub message: Option<String>,
    #[sea_orm(nullable)]
    pub review_note: Option<String>,
    #[sea_orm(nullable)]
    pub reviewed_at: Option<DateTimeUtc>,
    #[sea_orm(nullable)]
    pub reviewed_by: Option<UserId>,
    pub status: JoinRequestStatus,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "crate::entities::db::conversation::tenant::Entity",
        from = "Column::TenantId",
        to = "crate::entities::db::conversation::tenant::Column::Id"
    )]
    Tenant,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<crate::entities::db::conversation::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
