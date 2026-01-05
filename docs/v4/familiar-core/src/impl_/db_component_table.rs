//! Impl module for db_component_table types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for DbComponentTable

// Methods: table_name, is_hypertable, is_vector_table
impl DbComponentTable { pub fn table_name (& self) -> & 'static str { match self { Self :: FieldExcitations => "comp_field_excitations" , Self :: QuantumStates => "comp_quantum_states" , Self :: Content => "comp_content" , Self :: CognitiveOptics => "comp_cognitive_optics" , Self :: RelationalDynamics => "comp_relational_dynamics" , Self :: BondPhysics => "comp_bond_physics" , Self :: TaskDynamics => "comp_task_dynamics" , } } pub fn is_hypertable (& self) -> bool { matches ! (self , Self :: FieldExcitations) } pub fn is_vector_table (& self) -> bool { matches ! (self , Self :: QuantumStates) } }

