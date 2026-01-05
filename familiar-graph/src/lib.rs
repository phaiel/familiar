//! Schema dependency graph analysis for Familiar
//!
//! This crate provides graph analysis capabilities that were moved out of
//! familiar-schemas to keep the schema library pure and immutable.

pub mod loader;
pub mod analysis;
pub mod patterns;
pub mod classify;
pub mod diagnostics;

// Re-export key types from submodules
pub use analysis::{
    BoxedEdge, CycleHandling, FieldPath, FieldPathSegment, SccAnalysis, SccGroup,
    compute_scc_analysis, validate_boxed_edges,
};
pub use patterns::{
    JsonScalarKind, PropertyShape, PropertyTypeShape, SchemaShape, ObjectVariant,
    CodegenExtensions, detect_shape, detect_all_shapes,
};
pub use classify::{
    Classification, Classifier, EmitStrategy, EnumVariant, FieldDef, FieldType,
    TypeKind, UnionVariant, to_pascal_case, to_snake_case,
};
pub use diagnostics::{
    Diagnostics, DiagnosticCode, DiagnosticItem, Severity,
};
pub use loader::{
    SchemaGraph, SchemaId, SchemaNode, EdgeKind, NodeId,
    ClosureNode, SearchResult, LintWarning,
    GeneratedArtifact, ArtifactId, CodegenMeta, FieldRef,
};
