use serde::{Deserialize, Serialize};

/// The fundamental Entity Types recognized by The Heddle Classification Engine.
/// These map to the Symmetric Seven ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeddleEntityType {
    /// A specific event that happened in the past (Narrative/External Particle)
    Moment,
    /// A task or goal for the future (Operational/Intentional Particle)
    Intent, 
    /// A definition of a person, place, or concept (Definitional/Object)
    Thread, 
    /// A statement about the quality of a relationship (Relational/Connection)
    Bond,
    /// A recurring external pattern (External Wave)
    Motif,
    /// A recurring internal pattern (Internal Wave)
    Filament,
    /// An active thematic goal (Intentional Wave)
    Focus,
    /// An internal state snapshot (Internal Particle)
    Pulse,
}

