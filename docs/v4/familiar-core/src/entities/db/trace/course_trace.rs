//! SeaORM entity: CourseTrace
//! Table: course_trace
//!
//! Stores trace events for WebSocket backfill and analytics.
//! Consumed from Kafka familiar.trace topic by the Trace Projector.

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::CourseId;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "course_trace")]
pub struct Model {
    /// Course ID (part of composite primary key)
    #[sea_orm(primary_key, auto_increment = false)]
    pub course_id: CourseId,
    /// Sequence number for ordering (part of composite primary key)
    #[sea_orm(primary_key, auto_increment = false)]
    pub seq: i64,
    /// Span ID for trace hierarchy
    pub span_id: String,
    /// Parent span ID (optional)
    #[sea_orm(nullable)]
    pub parent_span_id: Option<String>,
    /// Trace kind (step, tool, thought, token, error, metric)
    pub kind: String,
    /// Trace status (started, in_progress, completed, failed, cancelled)
    pub status: String,
    /// When the trace event occurred
    pub occurred_at: DateTimeUtc,
    /// When this record was created
    pub created_at: DateTimeUtc,
    /// JSON payload with trace details
    #[sea_orm(column_type = "JsonBinary")]
    pub payload: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

