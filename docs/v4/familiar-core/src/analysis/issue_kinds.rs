//! Issue Types for Schema Analysis
//!
//! Defines all issue kinds, severities, and related types for the analyzer.

use serde::Serialize;
use std::path::PathBuf;

// ============================================================================
// Core Issue Types
// ============================================================================

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct AnalysisReport {
    pub issues: Vec<Issue>,
    pub stats: Stats,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Issue {
    pub file: PathBuf,
    pub line: usize,
    pub kind: IssueKind,
    pub severity: Severity,
    pub message: String,
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub enum IssueKind {
    /// Generated type exists but never used in code
    UnusedSchema { name: String },
    /// Hand-written type duplicates a generated one
    DuplicateType { name: String, generated_path: String },
    /// Core domain type in services should be in familiar-core
    MissingSchema { name: String },
    /// Raw primitive where typed wrapper should be used
    RawPrimitive { raw: String, suggested: String },
    /// Type looks like it should be a schema (suggestion to add to familiar-core)
    SuggestSchema { name: String, reason: String },
    /// Field suggests creating a new primitive type
    SuggestPrimitive { field: String, suggested: String },
    /// Type should be centralized in familiar-core based on naming pattern
    SuggestCentralize { name: String, category: String },
    
    // === OpenAPI / utoipa ===
    /// API type missing #[derive(ToSchema)] for OpenAPI
    MissingOpenApiDerive { name: String, has_serialize: bool },
    
    // === SeaORM / Entity ===
    /// Type should be a SeaORM entity (persistent domain object)
    SuggestEntity { name: String, reason: String, table_name: String },
    /// Entity missing required derives for SeaORM
    MissingEntityDerive { name: String, missing: Vec<String> },
    
    // === Consistency ===
    /// Type has inconsistent derives (e.g., TS but not ToSchema)
    InconsistentDerives { name: String, has: Vec<String>, missing: Vec<String> },
    
    // === Generation Sync ===
    MissingGeneration { type_name: String, expected_file: String },
    /// Generated file exists but has no source type (stale)
    StaleGeneration { file_name: String },
    
    // === Import Validation ===
    /// TypeScript import @familiar-core/X doesn't resolve
    BrokenImport { import_path: String, resolved_to: String },
    /// Local type definition should use schema from familiar-schemas
    /// `schema_exists` determines if schema is in familiar-schemas (use it) or not (add it)
    LocalTypeNotInSchema { 
        type_name: String, 
        language: String,  // "python", "typescript", etc.
        schema_exists: bool,
        schema_path: Option<String>,  // Path in familiar-schemas if exists
    },
    /// Legacy: Python import should use generated instead of local (deprecated, use LocalTypeNotInSchema)
    #[deprecated(note = "Use LocalTypeNotInSchema instead")]
    PythonImportNotGenerated { module: String, type_name: String, generated_alternative: String },
    
    // === Database Schema ===
    /// SQL table has no corresponding Rust entity
    TableWithoutEntity { table: String, suggested_struct: String },
    
    // === ECS Systems ===
    /// Function is a hecs-style System (takes &World, runs query loops)
    /// True systems are "governing forces" like decay_system, bond_resonance_system
    /// NOT trait-bounded helpers like fn process<T: HasPhysics>(...)
    SuggestSystem { 
        name: String, 
        components: Vec<String>, 
        location: String,
        /// Optional category from /// #[system(category = "physics")] annotation
        #[serde(skip_serializing_if = "Option::is_none")]
        category: Option<String>,
    },
    
    // === Laws (single-item operators) ===
    /// Function is a Law - trait-bounded single-item operator called by systems
    /// Laws are micro-operations like apply_decay, score_bond, compute_resonance
    /// They operate on ONE item at a time, systems fan them out across entities
    SuggestLaw {
        name: String,
        /// Trait bounds like ["HasDecay", "HasTimestamp"]
        trait_bounds: Vec<String>,
        /// Target type if known (e.g., "BondState")  
        target_type: Option<String>,
        location: String,
        /// Optional category from /// #[law(category = "decay")] annotation
        #[serde(skip_serializing_if = "Option::is_none")]
        category: Option<String>,
    },
    
    // === Schema Consolidation ===
    /// Type has created_at/updated_at that could use Timestamps component
    SuggestTimestamps {
        name: String,
        has_created: bool,
        has_updated: bool,
    },
    /// Response type could use ApiResult<T> wrapper
    SuggestApiResult {
        name: String,
        has_success: bool,
        has_error: bool,
    },
    /// Entity has id + tenant_id + timestamps that could use EntityMeta
    SuggestEntityMeta {
        name: String,
        id_field: String,
        has_tenant_id: bool,
        has_timestamps: bool,
    },
    /// Field uses raw Uuid where semantic primitive should be used
    SuggestSemanticPrimitive {
        struct_name: String,
        field_name: String,
        suggested_primitive: String,
    },
    
    // === Schema Decomposition ===
    /// Large struct that may need decomposition into components
    SuggestDecompose {
        name: String,
        field_count: usize,
        suggested_components: Vec<String>,
    },
    /// Multiple types share the same field pattern - candidate for component extraction
    SharedFieldPattern {
        field_names: Vec<String>,
        types_using: Vec<String>,
        suggested_component: String,
    },
    /// Type could implement Has* trait for composition
    SuggestHasTrait {
        type_name: String,
        trait_name: String,
        fields_covered: Vec<String>,
    },
    
    // === Duplicate Detection ===
    /// Same type name defined in multiple files (collision)
    DuplicateTypeName {
        name: String,
        locations: Vec<String>,
    },
    /// Type defines timestamps inline instead of using Timestamps component
    InlineTimestamps {
        name: String,
    },
    /// Entity type should use EntityMeta instead of inline fields
    MissingEntityMeta {
        name: String,
    },
    /// Exported type is defined but never used/imported
    UnusedExportedType {
        name: String,
        defined_at: String,
    },
    
    // === Kafka / Protobuf Codegen ===
    /// Kafka envelope type missing Protobuf message definition
    MissingProtobufSchema {
        type_name: String,
        file: String,
    },
    /// Kafka envelope missing kafka_key() method implementation
    MissingKafkaKey {
        type_name: String,
        file: String,
    },
    /// Using deprecated manual Kafka producer/consumer instead of generated
    ManualKafkaImplementation {
        component: String,
        file: String,
        suggested_replacement: String,
    },
    /// Schema registry subject doesn't follow naming convention
    InvalidSchemaRegistrySubject {
        subject: String,
        expected_pattern: String,
    },
    /// Kafka type not using ProtobufEnvelope trait
    MissingProtobufEnvelope {
        type_name: String,
        file: String,
    },
    /// Producer/Consumer using JSON when Protobuf should be used
    JsonSerializationInsteadOfProtobuf {
        component: String,
        file: String,
    },
    /// Codegen path violation - manual Kafka code in generated path
    CodegenPathViolation {
        file: String,
        reason: String,
    },
    // Note: KafkaMigrationOpportunity removed - now handled by ast-grep rules in rules/kafka/
    
    // === Communication Pattern Violations ===
    /// HTTP client used in worker service (should use Kafka)
    DirectHttpInWorker {
        component: String,
        file: String,
        suggested_alternative: String,
    },
    /// HTTP client struct field in worker (architectural violation)
    HttpClientInWorkerStruct {
        struct_name: String,
        file: String,
    },
    /// Direct inter-service call bypassing Kafka
    InterServiceBypassKafka {
        caller_service: String,
        target_service: String,
        method: String,
        file: String,
    },
    
    // === Database / SeaORM Compliance (Services) ===
    /// Direct sqlx::query() usage in services (should use TigerDataStore)
    DirectSqlxUsage {
        service: String,
        query_type: String,
    },
    /// Service bypassing TigerDataStore (direct database access)
    BypassingStore {
        service: String,
        pattern: String,
    },
    /// Legacy row mapping pattern (DbEntityRow, etc.)
    LegacyRowMapping {
        service: String,
        row_type: String,
    },
    /// Direct pool access in service (should use TigerDataStore methods)
    DirectPoolAccess {
        service: String,
    },
    /// Service defining database entities (should use familiar-core entities)
    EntityInService {
        service: String,
        entity_name: String,
    },
    
    // === Schema Graph Orphans ===
    /// Schema exists in familiar-schemas but not properly connected in schema graph
    /// Detected when schema has no incoming edges (not referenced by any other schema)
    OrphanSchema {
        /// Schema name (e.g., "CreateUserInput")
        schema_name: String,
        /// Path to schema file (e.g., "auth/CreateUserInput.schema.json")
        schema_path: String,
        /// Schema category/directory (e.g., "auth", "tools", "components")
        category: String,
        /// Whether the schema type is used in Rust/TS code (false = truly unused)
        used_in_code: bool,
        /// Locations where the type is referenced in code (file:line)
        usage_locations: Vec<String>,
        /// Recommended action to resolve the orphan status
        recommendation: OrphanRecommendation,
    },
    
    // === Missing Schemas (Rust types without JSON schema) ===
    /// Rust type exists in familiar-core/src/types/ but has no corresponding JSON schema
    MissingJsonSchema {
        /// Rust type name (e.g., "CourseStart")
        type_name: String,
        /// File where the type is defined
        source_file: String,
        /// Suggested schema path (e.g., "contracts/CourseStart.schema.json")
        suggested_schema_path: String,
    },
    
    // === Isolated Schemas (types/primitives with no refs) ===
    /// Type or primitive schema exists but is never referenced by other schemas
    IsolatedSchema {
        /// Schema name
        schema_name: String,
        /// Path to schema file
        schema_path: String,
        /// Category (types or primitives)
        category: String,
        /// Whether used in Rust code
        used_in_code: bool,
        /// Expected referencing schemas based on patterns
        expected_refs: Vec<String>,
    },
}

/// Recommendation for how to handle an orphan schema
#[derive(Debug, Clone, Serialize, schemars::JsonSchema, PartialEq, Eq)]
pub enum OrphanRecommendation {
    /// Schema is not used anywhere - safe to delete
    Delete,
    /// Schema is used in code but not connected in graph - add x-familiar-* extensions
    ConnectGraph,
    /// Schema is from deprecated system (e.g., Windmill) - mark as deprecated
    MarkDeprecated,
    /// Schema is an expected root node (infrastructure definition) - no action needed
    ExpectedRoot,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, schemars::JsonSchema)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Fix {
    pub description: String,
    pub replacement: Option<String>,
}

// ============================================================================
// Stats Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, schemars::JsonSchema)]
pub struct Stats {
    pub files_scanned: usize,
    /// Types in manifest (exported/generated types from familiar-core)
    pub types_exported: usize,
    /// Total types defined in code (all structs/enums found)
    pub types_defined: usize,
    /// Types defined within familiar-core
    pub types_in_familiar_core: usize,
    /// Types defined in services (should be minimal)
    pub types_in_services: usize,
    pub issues_found: usize,
    pub duration_ms: u64,
    /// Derive coverage tracking
    pub derive_coverage: DeriveCoverage,
    /// Entity candidates found
    pub entity_candidates: usize,
    /// ECS systems detected
    pub systems_detected: usize,
    /// Laws detected (single-item operators)
    pub laws_detected: usize,
    /// Decomposition candidates (large types)
    pub decomposition_candidates: usize,
    /// Shared field patterns detected
    pub shared_patterns: usize,
    /// Has* trait suggestions
    pub trait_suggestions: usize,
    /// Kafka/Protobuf issues found
    pub kafka_issues: usize,
    /// Manual Kafka implementations (deprecated)
    pub manual_kafka_impl: usize,
    /// Communication pattern violations (HTTP in workers, Kafka bypass)
    pub communication_violations: usize,
    /// HTTP clients found in worker services
    pub http_client_in_workers: usize,
    /// Direct inter-service calls bypassing Kafka
    pub kafka_bypass_detected: usize,
    /// Database/SeaORM compliance issues in services
    pub database_issues: usize,
    /// Direct sqlx usage in services
    pub direct_sqlx_usage: usize,
    /// Legacy row mapping patterns
    pub legacy_row_mapping: usize,
    /// Orphan schemas detected (not connected in graph)
    pub orphan_schemas: usize,
    /// Orphan schemas with ConnectGraph recommendation
    pub orphans_connect_graph: usize,
    /// Orphan schemas with Delete recommendation
    pub orphans_delete: usize,
    /// Orphan schemas with MarkDeprecated recommendation  
    pub orphans_deprecated: usize,
    /// Rust types without corresponding JSON schema
    pub missing_json_schemas: usize,
    /// Isolated types/primitives (not referenced by other schemas)
    pub isolated_schemas: usize,
}

#[derive(Debug, Clone, Default, Serialize, schemars::JsonSchema)]
pub struct DeriveCoverage {
    pub total_types: usize,
    pub with_serialize: usize,
    pub with_ts: usize,
    pub with_to_schema: usize,
    pub with_json_schema: usize,
}

/// Information about derives on a Rust type
#[derive(Debug, Clone, Default)]
pub struct DeriveInfo {
    pub serialize: bool,
    pub deserialize: bool,
    pub ts: bool,
    pub to_schema: bool,
    pub json_schema: bool,
    pub entity_model: bool,
    pub debug: bool,
    pub clone: bool,
}

