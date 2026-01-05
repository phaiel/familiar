//! SeaORM entity: ConsentRecord
//! Generated from: database/ConsentRecordModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{UserId, ConsentRecordId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ConsentType {
    #[sea_orm(string_value = "ai_processing")]
    AiProcessing,
    #[sea_orm(string_value = "analytics")]
    Analytics,
    #[sea_orm(string_value = "data_sharing")]
    DataSharing,
    #[sea_orm(string_value = "marketing_emails")]
    MarketingEmails,
    #[sea_orm(string_value = "privacy_policy")]
    PrivacyPolicy,
    #[sea_orm(string_value = "terms_of_service")]
    TermsOfService,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "consent_records")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: ConsentRecordId,
    pub consent_type: ConsentType,
    pub created_at: DateTimeUtc,
    pub granted: bool,
    #[sea_orm(nullable)]
    pub ip_address: Option<String>,
    #[sea_orm(nullable)]
    pub user_agent: Option<String>,
    pub user_id: UserId,
    #[sea_orm(nullable)]
    pub version: Option<String>,
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
