//! SeaORM entity: FamilyInvitation
//! Generated from: database/FamilyInvitationModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{TenantId, UserId, InvitationId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum InviteType {
    #[sea_orm(string_value = "code")]
    Code,
    #[sea_orm(string_value = "email")]
    Email,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum InviteRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "guest")]
    Guest,
    #[sea_orm(string_value = "member")]
    Member,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "family_invitations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: InvitationId,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub email: Option<String>,
    #[sea_orm(nullable)]
    pub expires_at: Option<DateTimeUtc>,
    #[sea_orm(nullable)]
    pub invite_code: Option<String>,
    pub invite_type: InviteType,
    #[sea_orm(nullable)]
    pub invited_by: Option<UserId>,
    #[sea_orm(nullable)]
    pub max_uses: Option<i32>,
    pub role: InviteRole,
    pub tenant_id: TenantId,
    pub use_count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::entities::db::conversation::tenant::Entity",
        from = "Column::TenantId",
        to = "crate::entities::db::conversation::tenant::Column::Id"
    )]
    Tenant,
}

impl Related<crate::entities::db::conversation::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
