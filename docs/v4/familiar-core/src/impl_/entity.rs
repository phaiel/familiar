//! Impl module for entity types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for Entity

// Trait impl: Related
impl Related < super :: user :: Entity > for Entity { fn to () -> RelationDef { Relation :: User . def () } }

// Trait impl: Related
impl Related < super :: session :: Entity > for Entity { fn to () -> RelationDef { Relation :: Sessions . def () } }

// Trait impl: Related
impl Related < super :: consent :: Entity > for Entity { fn to () -> RelationDef { Relation :: Consents . def () } }

// Trait impl: Related
impl Related < super :: user :: Entity > for Entity { fn to () -> RelationDef { Relation :: User . def () } }

// Trait impl: Related
impl Related < super :: user :: Entity > for Entity { fn to () -> RelationDef { Relation :: User . def () } }

// Trait impl: Related
impl Related < super :: super :: auth :: user :: Entity > for Entity { fn to () -> RelationDef { Relation :: User . def () } }

// Trait impl: Related
impl Related < super :: super :: conversation :: tenant :: Entity > for Entity { fn to () -> RelationDef { Relation :: Tenant . def () } }

// Trait impl: Related
impl Related < super :: entity_registry :: Entity > for Entity { fn to () -> RelationDef { Relation :: Entity . def () } }

// Trait impl: Related
impl Related < super :: entity_registry :: Entity > for Entity { fn to () -> RelationDef { Relation :: Entity . def () } }

// Trait impl: Related
impl Related < super :: field_excitation :: Entity > for Entity { fn to () -> RelationDef { Relation :: FieldExcitations . def () } }

// Trait impl: Related
impl Related < super :: quantum_state :: Entity > for Entity { fn to () -> RelationDef { Relation :: QuantumState . def () } }

// Trait impl: Related
impl Related < super :: content :: Entity > for Entity { fn to () -> RelationDef { Relation :: Content . def () } }

// Trait impl: Related
impl Related < super :: entity_registry :: Entity > for Entity { fn to () -> RelationDef { Relation :: Entity . def () } }

// Trait impl: Related
impl Related < super :: channel :: Entity > for Entity { fn to () -> RelationDef { Relation :: Channel . def () } }

// Trait impl: Related
impl Related < super :: tenant :: Entity > for Entity { fn to () -> RelationDef { Relation :: Tenant . def () } }

// Trait impl: Related
impl Related < super :: message :: Entity > for Entity { fn to () -> RelationDef { Relation :: Messages . def () } }

// Trait impl: Related
impl Related < super :: tenant_member :: Entity > for Entity { fn to () -> RelationDef { Relation :: Members . def () } }

// Trait impl: Related
impl Related < super :: channel :: Entity > for Entity { fn to () -> RelationDef { Relation :: Channels . def () } }

// Trait impl: Related
impl Related < super :: tenant :: Entity > for Entity { fn to () -> RelationDef { Relation :: Tenant . def () } }

// Trait impl: Related
impl Related < super :: tenant :: Entity > for Entity { fn to () -> RelationDef { Relation :: Tenant . def () } }

