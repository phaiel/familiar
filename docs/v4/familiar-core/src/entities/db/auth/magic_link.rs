//! SeaORM entity: MagicLink
//! Generated from: database/MagicLinkModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{MagicLinkId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum MagicLinkPurpose {
    #[sea_orm(string_value = "login")]
    Login,
    #[sea_orm(string_value = "password_reset")]
    PasswordReset,
    #[sea_orm(string_value = "signup")]
    Signup,
    #[sea_orm(string_value = "verify_email")]
    VerifyEmail,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "magic_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: MagicLinkId,
    pub created_at: DateTimeUtc,
    pub email: String,
    pub expires_at: DateTimeUtc,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    pub purpose: MagicLinkPurpose,
    pub token_hash: String,
    #[sea_orm(nullable)]
    pub used_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
}

impl ActiveModelBehavior for ActiveModel {}
