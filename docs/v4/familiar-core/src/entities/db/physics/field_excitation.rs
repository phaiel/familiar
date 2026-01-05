//! SeaORM entity: FieldExcitation
//! Generated from: database/FieldExcitationModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{EntityId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "comp_field_excitations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub entity_id: EntityId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub time: DateTimeUtc,
    pub amplitude: f64,
    pub energy: f64,
    pub pos_arousal: i64,
    pub pos_epistemic: i64,
    pub pos_valence: i64,
    pub temperature: f64,
    pub vel_arousal: i64,
    pub vel_epistemic: i64,
    pub vel_valence: i64,
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
