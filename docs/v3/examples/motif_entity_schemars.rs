// Example: Motif Entity using schemars (Rust-first approach)
// This solves the enum+const conflict and enables bidirectional validation

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// LEVEL 0-3: Generated from JSON Schema (keep existing approach)
// ============================================================================

pub type EntityId = Uuid;
pub type TenantId = Uuid;
pub type Timestamp = DateTime<Utc>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "A floating-point value normalized between 0.0 and 1.0")]
pub struct NormalizedValue(f64);

impl NormalizedValue {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ValidationError::OutOfRange {
                field: "NormalizedValue",
                min: 0.0,
                max: 1.0,
                actual: value,
            })
        }
    }
    
    pub fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexNumber {
    pub real: f64,
    pub imaginary: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DensityMatrix([[ComplexNumber; 2]; 2]);

pub type EntanglementMap = std::collections::HashMap<EntityId, NormalizedValue>;

#[derive(Debug)]
pub enum ValidationError {
    OutOfRange {
        field: &'static str,
        min: f64,
        max: f64,
        actual: f64,
    },
}

// ============================================================================
// LEVEL 4-5: Define in Rust with schemars (NEW APPROACH)
// ============================================================================

// Instead of enum with const, use a unit struct per entity type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Entity type for Motif (always 'Motif')")]
pub struct MotifEntityType;

impl MotifEntityType {
    pub const VALUE: &'static str = "Motif";
    
    pub fn as_str(&self) -> &'static str {
        Self::VALUE
    }
}

impl Default for MotifEntityType {
    fn default() -> Self {
        Self
    }
}

// Serialize as string "Motif"
impl serde::Serialize for MotifEntityType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(Self::VALUE)
    }
}

// Component: MotifContent
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Motif Content Component")]
#[schemars(description = "Defines the descriptive content of a Motif entity")]
pub struct MotifContent {
    #[schemars(description = "The type/category of this motif pattern")]
    pub motif_type: MotifType,
    
    #[schemars(description = "The thematic label for this motif")]
    pub theme: String,
    
    #[schemars(description = "The recurring pattern this motif represents")]
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Types of motif patterns")]
pub enum MotifType {
    Behavioral,
    Emotional,
    Cognitive,
    Social,
}

// Component: QuantumState
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Quantum State Component")]
#[schemars(description = "Manages the quantum properties of an entity")]
pub struct QuantumState {
    #[schemars(description = "The quantum density matrix representing superposition")]
    pub density_matrix: DensityMatrix,
    
    #[schemars(description = "Purity of quantum state (0.0 = mixed, 1.0 = pure)")]
    pub coherence_score: NormalizedValue,
    
    #[schemars(description = "Network of entangled entities")]
    pub entanglement_network: EntanglementMap,
}

// Component: ConsolidationState
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Consolidation State Component")]
#[schemars(description = "Tracks memory consolidation progress")]
pub struct ConsolidationState {
    #[schemars(description = "Rate of memory consolidation")]
    pub consolidation_rate: NormalizedValue,
    
    #[schemars(description = "Current memory strength")]
    pub memory_strength: NormalizedValue,
}

// Entity: Motif
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Motif Entity")]
#[schemars(description = "A quantum entity representing a recurring pattern of subjective experiences")]
pub struct Motif {
    // Base entity fields
    #[schemars(description = "Unique entity identifier")]
    pub entity_id: EntityId,
    
    #[schemars(description = "Tenant identifier")]
    pub tenant_id: TenantId,
    
    #[schemars(description = "Entity creation timestamp")]
    pub created_at: Timestamp,
    
    // Entity type (always "Motif" for this entity)
    #[serde(default)]
    #[schemars(description = "Entity type (always 'Motif')")]
    pub entity_type: MotifEntityType,
    
    // Components
    #[schemars(description = "Content component defining motif properties")]
    pub content: MotifContent,
    
    #[schemars(description = "Quantum state component for superposition")]
    pub quantum_state: QuantumState,
    
    #[schemars(description = "Memory consolidation state")]
    pub consolidation: ConsolidationState,
}

// ECS Component trait (entities can have multiple components)
pub trait Component {
    fn component_name(&self) -> &'static str;
}

impl Component for MotifContent {
    fn component_name(&self) -> &'static str {
        "MotifContent"
    }
}

impl Component for QuantumState {
    fn component_name(&self) -> &'static str {
        "QuantumState"
    }
}

impl Component for ConsolidationState {
    fn component_name(&self) -> &'static str {
        "ConsolidationState"
    }
}

// ============================================================================
// SCHEMA GENERATION & VALIDATION
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use jsonschema::JSONSchema;
    
    /// Generate JSON Schema from Rust type
    #[test]
    fn generate_motif_schema() {
        let schema = schema_for!(Motif);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        
        println!("Generated Motif JSON Schema:");
        println!("{}", schema_json);
        
        // Write to file for documentation/validation
        std::fs::write(
            "../../schemas/generated/Motif.schema.json",
            schema_json
        ).unwrap();
    }
    
    /// Round-trip validation: Rust → JSON → Validate → Rust
    #[test]
    fn test_motif_round_trip_validation() {
        // 1. Create Rust instance
        let motif = Motif {
            entity_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            created_at: Utc::now(),
            entity_type: MotifEntityType::default(),
            content: MotifContent {
                motif_type: MotifType::Emotional,
                theme: "stress-sleep".to_string(),
                pattern: "anxiety before bedtime".to_string(),
            },
            quantum_state: QuantumState {
                density_matrix: DensityMatrix([[
                    ComplexNumber { real: 1.0, imaginary: 0.0 },
                    ComplexNumber { real: 0.0, imaginary: 0.0 },
                ], [
                    ComplexNumber { real: 0.0, imaginary: 0.0 },
                    ComplexNumber { real: 0.0, imaginary: 0.0 },
                ]]),
                coherence_score: NormalizedValue::new(0.95).unwrap(),
                entanglement_network: std::collections::HashMap::new(),
            },
            consolidation: ConsolidationState {
                consolidation_rate: NormalizedValue::new(0.5).unwrap(),
                memory_strength: NormalizedValue::new(0.7).unwrap(),
            },
        };
        
        // 2. Serialize Rust to JSON
        let json_value = serde_json::to_value(&motif).unwrap();
        let json_str = serde_json::to_string_pretty(&json_value).unwrap();
        
        println!("Serialized Motif:");
        println!("{}", json_str);
        
        // 3. Get JSON Schema from Rust type
        let schema = schema_for!(Motif);
        let schema_value = serde_json::to_value(schema).unwrap();
        
        // 4. Validate JSON against schema
        let compiled_schema = JSONSchema::compile(&schema_value)
            .expect("Schema should compile");
        
        assert!(
            compiled_schema.is_valid(&json_value),
            "Rust instance must validate against its own schema"
        );
        
        // 5. Deserialize JSON back to Rust
        let motif2: Motif = serde_json::from_value(json_value)
            .expect("Should deserialize back to Rust");
        
        // 6. Verify round-trip preserves data
        assert_eq!(motif.entity_id, motif2.entity_id);
        assert_eq!(motif.tenant_id, motif2.tenant_id);
        assert_eq!(motif.content.theme, motif2.content.theme);
        assert_eq!(
            motif.quantum_state.coherence_score.get(),
            motif2.quantum_state.coherence_score.get()
        );
        
        println!("✅ Round-trip validation successful!");
    }
    
    /// Validate external JSON data against generated schema
    #[test]
    fn test_validate_external_json() {
        // JSON data from external source (API, database, etc.)
        let external_json = r#"{
            "entity_id": "550e8400-e29b-41d4-a716-446655440000",
            "tenant_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
            "created_at": "2025-01-06T12:00:00Z",
            "entity_type": "Motif",
            "content": {
                "motif_type": "Emotional",
                "theme": "anxiety",
                "pattern": "recurring worry"
            },
            "quantum_state": {
                "density_matrix": [
                    [
                        {"real": 1.0, "imaginary": 0.0},
                        {"real": 0.0, "imaginary": 0.0}
                    ],
                    [
                        {"real": 0.0, "imaginary": 0.0},
                        {"real": 0.0, "imaginary": 0.0}
                    ]
                ],
                "coherence_score": 0.8,
                "entanglement_network": {}
            },
            "consolidation": {
                "consolidation_rate": 0.3,
                "memory_strength": 0.6
            }
        }"#;
        
        let json_value: serde_json::Value = serde_json::from_str(external_json)
            .expect("Should parse JSON");
        
        // Get schema and validate
        let schema = schema_for!(Motif);
        let schema_value = serde_json::to_value(schema).unwrap();
        let compiled_schema = JSONSchema::compile(&schema_value)
            .expect("Schema should compile");
        
        let validation_result = compiled_schema.validate(&json_value);
        
        match validation_result {
            Ok(_) => println!("✅ External JSON validates against schema"),
            Err(errors) => {
                for error in errors {
                    eprintln!("Validation error: {}", error);
                    eprintln!("Instance path: {}", error.instance_path);
                }
                panic!("Validation failed");
            }
        }
        
        // Also verify we can deserialize
        let motif: Motif = serde_json::from_value(json_value)
            .expect("Should deserialize to Rust");
        
        assert_eq!(motif.content.theme, "anxiety");
        println!("✅ External JSON successfully deserialized to Rust");
    }
    
    /// Test that invalid data fails validation
    #[test]
    fn test_invalid_data_fails_validation() {
        let invalid_json = r#"{
            "entity_id": "not-a-uuid",
            "entity_type": "Motif"
        }"#;
        
        let json_value: serde_json::Value = serde_json::from_str(invalid_json).unwrap();
        
        let schema = schema_for!(Motif);
        let schema_value = serde_json::to_value(schema).unwrap();
        let compiled_schema = JSONSchema::compile(&schema_value).unwrap();
        
        // Should fail validation
        assert!(
            !compiled_schema.is_valid(&json_value),
            "Invalid data should fail validation"
        );
        
        println!("✅ Invalid data correctly rejected");
    }
}

// ============================================================================
// SCHEMA GENERATION BINARY
// ============================================================================

#[cfg(feature = "generate-schemas")]
fn main() {
    use std::fs;
    use std::path::Path;
    
    let output_dir = Path::new("docs/v3/schemas/generated");
    fs::create_dir_all(output_dir).unwrap();
    
    // Generate schema for Motif
    let schema = schema_for!(Motif);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write(
        output_dir.join("Motif.schema.json"),
        schema_json
    ).unwrap();
    
    println!("✅ Generated Motif.schema.json");
    
    // Could generate for other entities here
    // let thread_schema = schema_for!(Thread);
    // ...
}

#[cfg(not(feature = "generate-schemas"))]
fn main() {
    println!("Run with --features generate-schemas to generate JSON Schemas");
}

