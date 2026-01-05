//! SeaORM entity: ContentPayload
//! Generated from: database/ContentPayloadModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{EntityId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "comp_content")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub entity_id: EntityId,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    pub text_content: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entity_registry::Entity",
        from = "Column::EntityId",
        to = "super::entity_registry::Column::Id"
    )]
    EntityRegistry,
}

impl Related<super::entity_registry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EntityRegistry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
