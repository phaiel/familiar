//! SeaORM entity: AsyncTask
//! Table: async_tasks
//!
//! Tracks async task status for Kafka command processing.
//! Used by UI to poll for task completion.

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{TaskId, UserId, TenantId};

/// Task status enum
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, schemars::JsonSchema)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum TaskStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "queued")]
    Queued,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "async_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: TaskId,
    pub task_type: String,
    pub correlation_id: String,
    pub status: TaskStatus,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub input: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub output: Option<Json>,
    #[sea_orm(nullable)]
    pub error_message: Option<String>,
    #[sea_orm(default_value = 0)]
    pub attempt_count: i32,
    pub user_id: UserId,
    #[sea_orm(nullable)]
    pub tenant_id: Option<TenantId>,
    pub created_at: DateTimeUtc,
    #[sea_orm(nullable)]
    pub started_at: Option<DateTimeUtc>,
    #[sea_orm(nullable)]
    pub completed_at: Option<DateTimeUtc>,
    /// Optimistic locking version - incremented on each update
    /// Use with `update_with_version` to prevent lost updates
    #[sea_orm(default_value = 0)]
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::auth::user::Entity",
        from = "Column::UserId",
        to = "super::super::auth::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::super::conversation::tenant::Entity",
        from = "Column::TenantId",
        to = "super::super::conversation::tenant::Column::Id"
    )]
    Tenant,
}

impl Related<super::super::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::super::conversation::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

