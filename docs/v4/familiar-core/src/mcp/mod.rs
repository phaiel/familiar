//! MCP Server for Schema Registry
//!
//! Exposes the schema graph to AI agents via the Model Context Protocol (MCP).
//! Optimized for agent workflows: shallow composable responses, $id-based resolution,
//! and opinionated tools that turn graph knowledge into scaffolding actions.
//!
//! ## Architecture
//!
//! Graph infrastructure now lives in `familiar-schemas::graph` and is shared between
//! the MCP server and codegen. This ensures consistent behavior across both.
//!
//! - **SchemaGraph**: Primary data structure using petgraph for dependencies + cycles
//! - **HashMap indexes**: O(1) lookup by $id, path, or name
//! - **Minimal nodes**: No full schema by default, lazy-loaded on request
//! - **Bundle hash**: For caching and change detection
//!
//! ## Tools
//!
//! - Meta: `status`, `resolve`, `schema_raw`
//! - Query: `get_type`, `get_refs`, `get_dependents`, `search`, `list_kinds`
//! - Agent: `closure`, `imports_for`, `lint_unions`, `examples`, `services_for_schema`

// Legacy local graph module - being phased out in favor of familiar_schemas::graph
pub mod graph;
pub mod tools;

// Re-export from familiar-schemas (the new canonical location)
pub use familiar_schemas::graph::{
    SchemaGraph, SchemaNode, FieldRef, CodegenMeta, EdgeKind,
    SchemaId, NodeId, GeneratedArtifact, ArtifactId,
    ClosureNode, SearchResult, LintWarning,
};

// Re-export analysis types
pub use familiar_schemas::graph::{
    SccAnalysis, SccGroup, CycleHandling, BoxedEdge, FieldPathSegment,
    compute_scc_analysis,
};

// Re-export pattern types
pub use familiar_schemas::graph::{
    SchemaShape, JsonScalarKind, PropertyShape, PropertyTypeShape,
    detect_shape, detect_all_shapes,
};

// Re-export classification types
pub use familiar_schemas::graph::{
    Classification, TypeKind, EmitStrategy, FieldType, FieldDef, EnumVariant,
    Classifier, to_pascal_case, to_snake_case,
};

// Re-export diagnostics
pub use familiar_schemas::graph::{
    Diagnostics, DiagnosticCode, DiagnosticItem, Severity,
};

// Local tools remain here
pub use tools::SchemaTools;

