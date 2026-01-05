//! SeaORM entity: EntityRegistry
//! Generated from: database/EntityRegistryModel.schema.json
//! DO NOT EDIT - Regenerate with: cargo xtask codegen sea-entities

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use familiar_primitives::{EntityId, TenantId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[sea_orm(table_name = "entities")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: EntityId,
    pub created_at: DateTimeUtc,
    pub entity_type: String,
    pub tenant_id: TenantId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::field_excitation::Entity")]
    FieldExcitations,
    #[sea_orm(has_one = "super::quantum_state::Entity")]
    QuantumState,
    #[sea_orm(has_one = "super::content::Entity")]
    Content,
}

impl Related<super::field_excitation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FieldExcitations.def()
    }
}

impl Related<super::quantum_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::QuantumState.def()
    }
}

impl Related<super::content::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Content.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
