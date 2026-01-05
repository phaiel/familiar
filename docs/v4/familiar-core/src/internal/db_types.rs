//! Database table type enumeration and pool configuration

use serde::{Deserialize, Serialize};
use crate::primitives::{DbConnectionString, DbPoolSize};

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

/// Database pool configuration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DbPoolConfig {
    /// Database connection string
    pub connection: DbConnectionString,
    /// Maximum pool size
    pub max_connections: DbPoolSize,
    /// Minimum pool size
    pub min_connections: DbPoolSize,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
}

impl DbPoolConfig {
    pub fn new(connection: DbConnectionString) -> Self {
        Self {
            connection,
            max_connections: DbPoolSize::new(10).unwrap(),
            min_connections: DbPoolSize::new(1).unwrap(),
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }

    pub fn with_max_connections(mut self, size: u32) -> Result<Self, String> {
        self.max_connections = DbPoolSize::new(size)?;
        Ok(self)
    }

    pub fn with_min_connections(mut self, size: u32) -> Result<Self, String> {
        self.min_connections = DbPoolSize::new(size)?;
        Ok(self)
    }
}

impl Default for DbPoolConfig {
    fn default() -> Self {
        Self::new(DbConnectionString::default())
    }
}


