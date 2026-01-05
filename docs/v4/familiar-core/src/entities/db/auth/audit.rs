//! SeaORM entity: AuditLogEntry
//! Generated from: database/AuditLogEntryModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{UserId, AuditLogId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "audit_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: AuditLogId,
    pub action: String,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub error_message: Option<String>,
    #[sea_orm(nullable)]
    pub ip_address: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    #[sea_orm(nullable)]
    pub resource_id: Option<Uuid>,
    #[sea_orm(nullable)]
    pub resource_type: Option<String>,
    pub success: bool,
    #[sea_orm(nullable)]
    pub user_agent: Option<String>,
    #[sea_orm(nullable)]
    pub user_email: Option<String>,
    #[sea_orm(nullable)]
    pub user_id: Option<UserId>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
}

impl ActiveModelBehavior for ActiveModel {}
