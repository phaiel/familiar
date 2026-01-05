//! SeaORM entity: AuthSession
//! Generated from: database/AuthSessionModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{SessionId, UserId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "auth_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: SessionId,
    pub created_at: DateTimeUtc,
    pub expires_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub ip_address: Option<String>,
    pub token_hash: String,
    #[sea_orm(nullable)]
    pub user_agent: Option<String>,
    pub user_id: UserId,
    #[sea_orm(default_value = 0)]
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
