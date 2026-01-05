//! Database table type enumeration for the entity registry

use serde::{Deserialize, Serialize};

/// Entity types stored in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum DbEntityTable {
    Pulse,
    Thread,
    Bond,
    Moment,
    Intent,
    Focus,
    Motif,
    Filament,
}

impl DbEntityTable {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pulse => "Pulse",
            Self::Thread => "Thread",
            Self::Bond => "Bond",
            Self::Moment => "Moment",
            Self::Intent => "Intent",
            Self::Focus => "Focus",
            Self::Motif => "Motif",
            Self::Filament => "Filament",
        }
    }
}

impl std::fmt::Display for DbEntityTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Component tables (hypertables for time-series, regular for static)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
pub enum DbComponentTable {
    /// Field excitations (hypertable - time-series physics state)
    FieldExcitations,
    /// Quantum states (vector table for embeddings)
    QuantumStates,
    /// Content payloads
    Content,
    /// Cognitive optics
    CognitiveOptics,
    /// Relational dynamics
    RelationalDynamics,
    /// Bond physics
    BondPhysics,
    /// Task dynamics
    TaskDynamics,
}

impl DbComponentTable {
    pub fn table_name(&self) -> &'static str {
        match self {
            Self::FieldExcitations => "comp_field_excitations",
            Self::QuantumStates => "comp_quantum_states",
            Self::Content => "comp_content",
            Self::CognitiveOptics => "comp_cognitive_optics",
            Self::RelationalDynamics => "comp_relational_dynamics",
            Self::BondPhysics => "comp_bond_physics",
            Self::TaskDynamics => "comp_task_dynamics",
        }
    }

    pub fn is_hypertable(&self) -> bool {
        matches!(self, Self::FieldExcitations)
    }

    pub fn is_vector_table(&self) -> bool {
        matches!(self, Self::QuantumStates)
    }
}

