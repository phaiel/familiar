//! SeaORM entity: QuantumState
//! Generated from: database/QuantumStateModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{EntityId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "comp_quantum_states")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub entity_id: EntityId,
    pub coherence: f64,
    #[sea_orm(nullable)]
    pub frequency: Option<f64>,
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
