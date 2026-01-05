//! Schema Dependency Graph for MCP
//!
//! Primary data structure using petgraph for $ref/allOf/anyOf/oneOf dependencies.
//! Handles cycles correctly (SCCs). Provides fast lookup via HashMap indexes.

use include_dir::Dir;
use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Canonical schema identifier (the $id field)
pub type SchemaId = String;

/// Unique identifier for a graph node (schema or artifact)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeId {
    /// Schema node: "primitives/TenantId.schema.json"
    Schema(SchemaId),
    /// Artifact node: "rust:TenantId" or "typescript:TenantId"
    Artifact { lang: String, type_name: String },
}

impl NodeId {
    pub fn schema(id: impl Into<String>) -> Self {
        NodeId::Schema(id.into())
    }
    
    pub fn artifact(lang: impl Into<String>, type_name: impl Into<String>) -> Self {
        NodeId::Artifact { lang: lang.into(), type_name: type_name.into() }
    }
    
    pub fn as_schema(&self) -> Option<&str> {
        match self {
            NodeId::Schema(id) => Some(id),
            _ => None,
        }
    }
    
    pub fn as_artifact(&self) -> Option<(&str, &str)> {
        match self {
            NodeId::Artifact { lang, type_name } => Some((lang, type_name)),
            _ => None,
        }
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeId::Schema(id) => write!(f, "schema:{}", id),
            NodeId::Artifact { lang, type_name } => write!(f, "artifact:{}:{}", lang, type_name),
        }
    }
}

/// Types of edges in the schema dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Standard $ref dependency
    Ref,
    /// allOf composition (inheritance/mixin)
    AllOf,
    /// oneOf discriminated union variant
    OneOf,
    /// anyOf union type option  
    AnyOf,
    /// items array element type
    Items,
    /// additionalProperties map value type
    AdditionalProperties,
    /// Property field type
    Property,
    /// Schema generates this artifact (Schema → Artifact edge)
    GeneratesTo,
    /// Artifact depends on schema (Artifact → Schema edge for cross-file deps)
    DependsOn,
}

/// Unique artifact identifier: "lang:type_name" (e.g., "rust:TenantId")
pub type ArtifactId = String;

/// Generated artifact location for a specific language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    /// Unique artifact ID (lang:type_name)
    #[serde(default)]
    pub id: ArtifactId,
    /// Language: "rust", "typescript", "python"
    pub lang: String,
    /// Relative file path from workspace root
    pub file: PathBuf,
    /// Line number where the type definition starts (1-indexed)
    pub line: u32,
    /// Generated type name (may differ from schema title due to naming conventions)
    pub type_name: String,
    /// Type kind: "struct", "enum", "newtype", "type_alias"
    pub type_kind: String,
}

impl GeneratedArtifact {
    /// Create artifact ID from lang and type_name
    pub fn make_id(lang: &str, type_name: &str) -> ArtifactId {
        format!("{}:{}", lang, type_name)
    }
    
    /// Get or generate the artifact ID
    pub fn artifact_id(&self) -> ArtifactId {
        if self.id.is_empty() {
            Self::make_id(&self.lang, &self.type_name)
        } else {
            self.id.clone()
        }
    }
}

/// Codegen metadata extracted from x-familiar-* extensions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodegenMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_repr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flatten: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_none: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newtype: Option<bool>,
    /// Generated artifacts for this schema (populated by codegen)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<GeneratedArtifact>,
}

/// Reference to a field's type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ty_ref: Option<SchemaId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Minimal schema node data (no full schema by default)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaNode {
    /// Canonical $id
    pub id: SchemaId,
    /// File path relative to schema root
    pub path: PathBuf,
    /// Schema title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// x-familiar-kind
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// x-familiar-service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// Field references (name + type ref or inline kind)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldRef>,
    /// Codegen metadata from x-familiar-* extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codegen: Option<CodegenMeta>,
    /// Graph node index for fast lookup
    #[serde(skip)]
    pub node_idx: Option<NodeIndex>,
}

/// The schema dependency graph
pub struct SchemaGraph {
    /// Primary graph structure - DAG with potential cycles
    graph: DiGraph<SchemaId, EdgeKind>,
    
    /// Minimal node data indexed by $id
    schemas: HashMap<SchemaId, SchemaNode>,
    
    /// Index: file path -> $id
    by_path: HashMap<PathBuf, SchemaId>,
    
    /// Index: name (title or filename) -> list of $ids (names can collide!)
    by_name: HashMap<String, Vec<SchemaId>>,
    
    /// Raw JSON stored separately (lazy loaded on request)
    raw_schemas: HashMap<SchemaId, serde_json::Value>,
    
    /// Node index lookup: $id -> NodeIndex
    node_indices: HashMap<SchemaId, NodeIndex>,
    
    /// Bundle hash for caching/determinism
    pub bundle_hash: String,
    
    /// Strongly connected components (circular ref groups)
    scc_groups: Vec<Vec<SchemaId>>,
    
    // ========== Artifact Graph Indexes (O(1) lookups) ==========
    
    /// All artifacts indexed by ID
    artifacts: HashMap<ArtifactId, GeneratedArtifact>,
    
    /// Index: schema_id -> artifact_ids (one schema can generate multiple artifacts: rust, ts, py)
    schema_to_artifacts: HashMap<SchemaId, Vec<ArtifactId>>,
    
    /// Index: artifact_id -> schema_id (reverse lookup)
    artifact_to_schema: HashMap<ArtifactId, SchemaId>,
    
    /// Index: file_path -> artifact_ids (all types in a file)
    file_to_artifacts: HashMap<PathBuf, Vec<ArtifactId>>,
    
    /// Index: lang -> artifact_ids (all artifacts for a language)
    lang_to_artifacts: HashMap<String, Vec<ArtifactId>>,
}

impl SchemaGraph {
    /// Load schemas from a directory with fast parse
    #[inline]
    pub fn from_directory(schema_dir: &Path) -> anyhow::Result<Self> {
        // Pre-count files for capacity estimation
        let file_count = WalkDir::new(schema_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .count();
        
        let schema_count = file_count.max(100); // Minimum 100 for small dirs
        
        // Pre-size all collections
        let mut graph = DiGraph::with_capacity(schema_count, schema_count * 3);
        let mut schemas = HashMap::with_capacity(schema_count);
        let mut by_path = HashMap::with_capacity(schema_count);
        let mut by_name: HashMap<String, Vec<SchemaId>> = HashMap::with_capacity(schema_count);
        let mut raw_schemas = HashMap::with_capacity(schema_count);
        let mut node_indices = HashMap::with_capacity(schema_count);
        let mut hasher = Sha256::new();
        
        // Collect all refs for later edge creation
        let mut pending_refs: Vec<(SchemaId, String, EdgeKind)> = Vec::with_capacity(schema_count * 3);
        
        // Phase 1: Parse all schemas, extract metadata, collect refs
        for entry in WalkDir::new(schema_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            
            let content = fs::read_to_string(path)?;
            hasher.update(content.as_bytes());
            
            let json: serde_json::Value = serde_json::from_str(&content)?;
            let relative_path = path.strip_prefix(schema_dir)?.to_path_buf();
            
            // Extract $id or generate from path
            let id = json.get("$id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| relative_path.to_string_lossy().to_string());
            
            // Extract title
            let title = json.get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Extract x-familiar-kind
            let kind = json.get("x-familiar-kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Extract x-familiar-service
            let service = json.get("x-familiar-service")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Extract codegen metadata
            let codegen = Self::extract_codegen(&json);
            
            // Extract fields from properties
            let fields = Self::extract_fields(&json, &relative_path);
            
            // Collect all $refs for edge creation
            Self::collect_refs(&json, &id, &relative_path, &mut pending_refs);
            
            // Add node to graph
            let node_idx = graph.add_node(id.clone());
            
            let node = SchemaNode {
                id: id.clone(),
                path: relative_path.clone(),
                title: title.clone(),
                kind,
                service,
                fields,
                codegen,
                node_idx: Some(node_idx),
            };
            
            // Index by path
            by_path.insert(relative_path, id.clone());
            
            // Index by name (title or filename)
            let name = title.unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .replace(".schema", "")
            });
            by_name.entry(name).or_default().push(id.clone());
            
            // Store node and raw schema
            node_indices.insert(id.clone(), node_idx);
            schemas.insert(id.clone(), node);
            raw_schemas.insert(id, json);
        }
        
        // Phase 2: Create edges from collected refs
        for (from_id, ref_target, edge_kind) in pending_refs {
            let to_id = Self::resolve_ref(&ref_target, &from_id, &schemas, &by_path);
            
            if let (Some(&from_idx), Some(to_id)) = (node_indices.get(&from_id), to_id) {
                if let Some(&to_idx) = node_indices.get(to_id) {
                    graph.add_edge(from_idx, to_idx, edge_kind);
                }
            }
        }
        
        // Compute bundle hash
        let bundle_hash = format!("{:x}", hasher.finalize());
        
        // Compute SCCs (strongly connected components for circular refs)
        let scc_indices = kosaraju_scc(&graph);
        let scc_groups: Vec<Vec<SchemaId>> = scc_indices
            .into_iter()
            .filter(|scc| scc.len() > 1) // Only keep actual cycles
            .map(|scc| {
                scc.into_iter()
                    .filter_map(|idx| graph.node_weight(idx).cloned())
                    .collect()
            })
            .collect();
        
        Ok(Self {
            graph,
            schemas,
            by_path,
            by_name,
            raw_schemas,
            node_indices,
            bundle_hash,
            scc_groups,
            // Artifact indexes - populated later via load_artifact_indexes()
            artifacts: HashMap::new(),
            schema_to_artifacts: HashMap::new(),
            artifact_to_schema: HashMap::new(),
            file_to_artifacts: HashMap::new(),
            lang_to_artifacts: HashMap::new(),
        })
    }
    
    /// Load schemas from embedded directory (compiled into binary via include_dir!)
    /// 
    /// This is the preferred method for the MCP - it uses schemas embedded at compile time,
    /// ensuring the MCP always serves the exact schemas familiar-core was built with.
    #[inline]
    pub fn from_embedded(embedded_dir: &'static Dir<'static>) -> anyhow::Result<Self> {
        // Collect all JSON files first to know capacity
        let mut files_to_process: Vec<_> = Vec::with_capacity(512); // Typical schema count
        Self::collect_embedded_files(embedded_dir, &mut files_to_process);
        
        let schema_count = files_to_process.len();
        
        // Pre-size all collections based on expected schema count
        let mut graph = DiGraph::with_capacity(schema_count, schema_count * 3);
        let mut schemas = HashMap::with_capacity(schema_count);
        let mut by_path = HashMap::with_capacity(schema_count);
        let mut by_name: HashMap<String, Vec<SchemaId>> = HashMap::with_capacity(schema_count);
        let mut raw_schemas = HashMap::with_capacity(schema_count);
        let mut node_indices = HashMap::with_capacity(schema_count);
        let mut hasher = Sha256::new();
        
        // Collect all refs for later edge creation (estimate ~3 refs per schema)
        let mut pending_refs: Vec<(SchemaId, String, EdgeKind)> = Vec::with_capacity(schema_count * 3);
        
        // Phase 1: Parse all schemas, extract metadata, collect refs
        for (path, content) in files_to_process {
            hasher.update(content.as_bytes());
            
            let json: serde_json::Value = match serde_json::from_str(content) {
                Ok(j) => j,
                Err(_) => continue, // Skip invalid JSON
            };
            let relative_path = path.to_path_buf();
            
            // Extract $id or generate from path
            let id = json.get("$id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| relative_path.to_string_lossy().to_string());
            
            // Extract title
            let title = json.get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Extract x-familiar-kind
            let kind = json.get("x-familiar-kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Extract x-familiar-service
            let service = json.get("x-familiar-service")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Extract codegen metadata
            let codegen = Self::extract_codegen(&json);
            
            // Extract fields from properties
            let fields = Self::extract_fields(&json, &relative_path);
            
            // Collect all $refs for edge creation
            Self::collect_refs(&json, &id, &relative_path, &mut pending_refs);
            
            // Add node to graph
            let node_idx = graph.add_node(id.clone());
            
            let node = SchemaNode {
                id: id.clone(),
                path: relative_path.clone(),
                title: title.clone(),
                kind,
                service,
                fields,
                codegen,
                node_idx: Some(node_idx),
            };
            
            // Index by path
            by_path.insert(relative_path, id.clone());
            
            // Index by name (title or filename)
            let name = title.unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .replace(".schema", "")
            });
            by_name.entry(name).or_default().push(id.clone());
            
            // Store node and raw schema
            node_indices.insert(id.clone(), node_idx);
            schemas.insert(id.clone(), node);
            raw_schemas.insert(id, json);
        }
        
        // Phase 2: Create edges from collected refs
        for (from_id, ref_target, edge_kind) in pending_refs {
            let to_id = Self::resolve_ref(&ref_target, &from_id, &schemas, &by_path);
            
            if let (Some(&from_idx), Some(to_id)) = (node_indices.get(&from_id), to_id) {
                if let Some(&to_idx) = node_indices.get(to_id) {
                    graph.add_edge(from_idx, to_idx, edge_kind);
                }
            }
        }
        
        // Compute bundle hash
        let bundle_hash = format!("{:x}", hasher.finalize());
        
        // Compute SCCs (strongly connected components for circular refs)
        let scc_indices = kosaraju_scc(&graph);
        let scc_groups: Vec<Vec<SchemaId>> = scc_indices
            .into_iter()
            .filter(|scc| scc.len() > 1) // Only keep actual cycles
            .map(|scc| {
                scc.into_iter()
                    .filter_map(|idx| graph.node_weight(idx).cloned())
                    .collect()
            })
            .collect();
        
        Ok(Self {
            graph,
            schemas,
            by_path,
            by_name,
            raw_schemas,
            node_indices,
            bundle_hash,
            scc_groups,
            // Artifact indexes - populated later via load_artifact_indexes()
            artifacts: HashMap::new(),
            schema_to_artifacts: HashMap::new(),
            artifact_to_schema: HashMap::new(),
            file_to_artifacts: HashMap::new(),
            lang_to_artifacts: HashMap::new(),
        })
    }
    
    /// Recursively collect all JSON files from an embedded directory
    fn collect_embedded_files<'a>(
        dir: &'a Dir<'static>,
        files: &mut Vec<(&'a Path, &'a str)>,
    ) {
        // Collect files in this directory
        for file in dir.files() {
            let path = file.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(content) = file.contents_utf8() {
                    files.push((path, content));
                }
            }
        }
        
        // Recurse into subdirectories
        for subdir in dir.dirs() {
            Self::collect_embedded_files(subdir, files);
        }
    }
    
    /// Extract codegen metadata from x-familiar-* extensions
    fn extract_codegen(json: &serde_json::Value) -> Option<CodegenMeta> {
        let mut meta = CodegenMeta::default();
        let mut has_any = false;
        
        if let Some(v) = json.get("x-familiar-enum-repr").and_then(|v| v.as_str()) {
            meta.enum_repr = Some(v.to_string());
            has_any = true;
        }
        if let Some(v) = json.get("x-familiar-discriminator").and_then(|v| v.as_str()) {
            meta.discriminator = Some(v.to_string());
            has_any = true;
        }
        if let Some(v) = json.get("x-familiar-content").and_then(|v| v.as_str()) {
            meta.content = Some(v.to_string());
            has_any = true;
        }
        if let Some(v) = json.get("x-familiar-casing").and_then(|v| v.as_str()) {
            meta.casing = Some(v.to_string());
            has_any = true;
        }
        if let Some(v) = json.get("x-familiar-flatten").and_then(|v| v.as_bool()) {
            meta.flatten = Some(v);
            has_any = true;
        }
        if let Some(v) = json.get("x-familiar-skip-none").and_then(|v| v.as_bool()) {
            meta.skip_none = Some(v);
            has_any = true;
        }
        if let Some(v) = json.get("x-familiar-newtype").and_then(|v| v.as_bool()) {
            meta.newtype = Some(v);
            has_any = true;
        }
        
        if has_any { Some(meta) } else { None }
    }
    
    /// Extract fields from properties
    fn extract_fields(json: &serde_json::Value, current_path: &Path) -> Vec<FieldRef> {
        let mut fields = Vec::new();
        
        let required: HashSet<String> = json.get("required")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        if let Some(props) = json.get("properties").and_then(|v| v.as_object()) {
            for (name, prop) in props {
                let ty_ref = prop.get("$ref")
                    .and_then(|v| v.as_str())
                    .map(|s| Self::normalize_ref(s, current_path));
                
                let inline_kind = if ty_ref.is_none() {
                    prop.get("type").and_then(|v| v.as_str()).map(String::from)
                } else {
                    None
                };
                
                fields.push(FieldRef {
                    name: name.clone(),
                    ty_ref,
                    inline_kind,
                    required: Some(required.contains(name)),
                });
            }
        }
        
        fields
    }
    
    /// Normalize a $ref to a canonical path
    fn normalize_ref(ref_str: &str, current_path: &Path) -> String {
        if ref_str.starts_with('#') {
            // Local ref
            return ref_str.to_string();
        }
        
        if ref_str.starts_with("http://") || ref_str.starts_with("https://") {
            // Absolute URL
            return ref_str.to_string();
        }
        
        // Relative path - resolve against current file
        let parent = current_path.parent().unwrap_or(Path::new(""));
        let resolved = parent.join(ref_str);
        
        // Normalize the path (remove ../ etc)
        let mut components = Vec::new();
        for component in resolved.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::Normal(s) => {
                    components.push(s.to_string_lossy().to_string());
                }
                _ => {}
            }
        }
        
        components.join("/")
    }
    
    /// Collect all $refs from a schema
    fn collect_refs(
        json: &serde_json::Value,
        schema_id: &str,
        current_path: &Path,
        refs: &mut Vec<(SchemaId, String, EdgeKind)>,
    ) {
        match json {
            serde_json::Value::Object(obj) => {
                // Direct $ref
                if let Some(ref_val) = obj.get("$ref").and_then(|v| v.as_str()) {
                    if !ref_val.starts_with('#') {
                        refs.push((
                            schema_id.to_string(),
                            Self::normalize_ref(ref_val, current_path),
                            EdgeKind::Ref,
                        ));
                    }
                }
                
                // allOf
                if let Some(arr) = obj.get("allOf").and_then(|v| v.as_array()) {
                    for item in arr {
                        if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                            if !ref_val.starts_with('#') {
                                refs.push((
                                    schema_id.to_string(),
                                    Self::normalize_ref(ref_val, current_path),
                                    EdgeKind::AllOf,
                                ));
                            }
                        }
                        Self::collect_refs(item, schema_id, current_path, refs);
                    }
                }
                
                // oneOf
                if let Some(arr) = obj.get("oneOf").and_then(|v| v.as_array()) {
                    for item in arr {
                        if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                            if !ref_val.starts_with('#') {
                                refs.push((
                                    schema_id.to_string(),
                                    Self::normalize_ref(ref_val, current_path),
                                    EdgeKind::OneOf,
                                ));
                            }
                        }
                        Self::collect_refs(item, schema_id, current_path, refs);
                    }
                }
                
                // anyOf
                if let Some(arr) = obj.get("anyOf").and_then(|v| v.as_array()) {
                    for item in arr {
                        if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                            if !ref_val.starts_with('#') {
                                refs.push((
                                    schema_id.to_string(),
                                    Self::normalize_ref(ref_val, current_path),
                                    EdgeKind::AnyOf,
                                ));
                            }
                        }
                        Self::collect_refs(item, schema_id, current_path, refs);
                    }
                }
                
                // items (array type)
                if let Some(items) = obj.get("items") {
                    if let Some(ref_val) = items.get("$ref").and_then(|v| v.as_str()) {
                        if !ref_val.starts_with('#') {
                            refs.push((
                                schema_id.to_string(),
                                Self::normalize_ref(ref_val, current_path),
                                EdgeKind::Items,
                            ));
                        }
                    }
                    Self::collect_refs(items, schema_id, current_path, refs);
                }
                
                // additionalProperties (map type)
                if let Some(add_props) = obj.get("additionalProperties") {
                    if let Some(ref_val) = add_props.get("$ref").and_then(|v| v.as_str()) {
                        if !ref_val.starts_with('#') {
                            refs.push((
                                schema_id.to_string(),
                                Self::normalize_ref(ref_val, current_path),
                                EdgeKind::AdditionalProperties,
                            ));
                        }
                    }
                    Self::collect_refs(add_props, schema_id, current_path, refs);
                }
                
                // properties
                if let Some(props) = obj.get("properties").and_then(|v| v.as_object()) {
                    for (_name, prop) in props {
                        if let Some(ref_val) = prop.get("$ref").and_then(|v| v.as_str()) {
                            if !ref_val.starts_with('#') {
                                refs.push((
                                    schema_id.to_string(),
                                    Self::normalize_ref(ref_val, current_path),
                                    EdgeKind::Property,
                                ));
                            }
                        }
                        Self::collect_refs(prop, schema_id, current_path, refs);
                    }
                }
                
                // Recurse into other object values
                for (key, value) in obj {
                    if !["$ref", "allOf", "oneOf", "anyOf", "items", "additionalProperties", "properties"].contains(&key.as_str()) {
                        Self::collect_refs(value, schema_id, current_path, refs);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    Self::collect_refs(item, schema_id, current_path, refs);
                }
            }
            _ => {}
        }
    }
    
    /// Resolve a ref target to a schema $id
    fn resolve_ref<'a>(
        ref_target: &str,
        _from_id: &str,
        schemas: &'a HashMap<SchemaId, SchemaNode>,
        by_path: &'a HashMap<PathBuf, SchemaId>,
    ) -> Option<&'a SchemaId> {
        // Try as direct $id match
        if schemas.contains_key(ref_target) {
            return schemas.get(ref_target).map(|n| &n.id);
        }
        
        // Try as path match
        let path = PathBuf::from(ref_target);
        if let Some(id) = by_path.get(&path) {
            return Some(id);
        }
        
        // Try normalized path
        let normalized = ref_target.replace(".schema.json", ".schema.json");
        if let Some(id) = by_path.get(&PathBuf::from(&normalized)) {
            return Some(id);
        }
        
        None
    }
    
    // ========== Public API ==========
    
    /// Get schema count
    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }
    
    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    
    /// Get SCC count (number of circular ref groups)
    pub fn scc_count(&self) -> usize {
        self.scc_groups.len()
    }
    
    /// Get SCC groups
    pub fn scc_groups(&self) -> &[Vec<SchemaId>] {
        &self.scc_groups
    }
    
    /// Resolve a query (name/path/id) to canonical $id
    pub fn resolve(&self, query: &str) -> Option<&SchemaId> {
        // Try as direct $id
        if self.schemas.contains_key(query) {
            return Some(&self.schemas.get(query)?.id);
        }
        
        // Try as path
        let path = PathBuf::from(query);
        if let Some(id) = self.by_path.get(&path) {
            return Some(id);
        }
        
        // Try as name (return first match)
        if let Some(ids) = self.by_name.get(query) {
            return ids.first();
        }
        
        // Try case-insensitive name match
        let query_lower = query.to_lowercase();
        for (name, ids) in &self.by_name {
            if name.to_lowercase() == query_lower {
                return ids.first();
            }
        }
        
        None
    }
    
    /// Get schema node by $id
    pub fn get(&self, id: &str) -> Option<&SchemaNode> {
        self.schemas.get(id)
    }
    
    /// Get raw JSON schema by $id
    pub fn get_raw(&self, id: &str) -> Option<&serde_json::Value> {
        self.raw_schemas.get(id)
    }
    
    /// Get immediate outgoing refs (dependencies)
    pub fn refs_out(&self, id: &str) -> Vec<&SchemaId> {
        let Some(&node_idx) = self.node_indices.get(id) else {
            return Vec::new();
        };
        
        self.graph
            .edges_directed(node_idx, Direction::Outgoing)
            .filter_map(|e| self.graph.node_weight(e.target()))
            .collect()
    }
    
    /// Get immediate incoming refs (dependents)
    pub fn refs_in(&self, id: &str) -> Vec<&SchemaId> {
        let Some(&node_idx) = self.node_indices.get(id) else {
            return Vec::new();
        };
        
        self.graph
            .edges_directed(node_idx, Direction::Incoming)
            .filter_map(|e| self.graph.node_weight(e.source()))
            .collect()
    }
    
    /// Get transitive closure (all deps or dependents)
    pub fn closure(
        &self,
        id: &str,
        direction: Direction,
        max_depth: Option<usize>,
    ) -> Vec<ClosureNode> {
        let Some(&start_idx) = self.node_indices.get(id) else {
            return Vec::new();
        };
        
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![(start_idx, 0usize)];
        
        // Track which SCCs we've seen
        let scc_set: HashSet<&SchemaId> = self.scc_groups
            .iter()
            .flatten()
            .collect();
        
        while let Some((node_idx, depth)) = stack.pop() {
            if let Some(max) = max_depth {
                if depth > max {
                    continue;
                }
            }
            
            if !visited.insert(node_idx) {
                continue;
            }
            
            if node_idx != start_idx {
                if let Some(node_id) = self.graph.node_weight(node_idx) {
                    result.push(ClosureNode {
                        id: node_id.clone(),
                        depth,
                        scc_boundary: scc_set.contains(node_id),
                    });
                }
            }
            
            let edges = self.graph.edges_directed(node_idx, direction);
            for edge in edges {
                let next = match direction {
                    Direction::Outgoing => edge.target(),
                    Direction::Incoming => edge.source(),
                };
                stack.push((next, depth + 1));
            }
        }
        
        // Sort by depth
        result.sort_by_key(|n| n.depth);
        result
    }
    
    /// Search schemas by name (fuzzy)
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;
        
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(i64, &SchemaNode)> = Vec::new();
        
        for node in self.schemas.values() {
            // Match against title
            if let Some(title) = &node.title {
                if let Some(score) = matcher.fuzzy_match(title, query) {
                    results.push((score, node));
                    continue;
                }
            }
            
            // Match against filename
            let filename = node.path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if let Some(score) = matcher.fuzzy_match(filename, query) {
                results.push((score, node));
            }
        }
        
        // Sort by score descending
        results.sort_by(|a, b| b.0.cmp(&a.0));
        
        results
            .into_iter()
            .take(limit)
            .map(|(score, node)| SearchResult {
                id: node.id.clone(),
                title: node.title.clone(),
                kind: node.kind.clone(),
                path: node.path.clone(),
                score,
            })
            .collect()
    }
    
    /// List all schemas by x-familiar-kind
    pub fn list_by_kind(&self, kind: &str) -> Vec<&SchemaId> {
        self.schemas
            .values()
            .filter(|n| n.kind.as_deref() == Some(kind))
            .map(|n| &n.id)
            .collect()
    }
    
    /// Get all unique kinds
    pub fn all_kinds(&self) -> Vec<String> {
        let mut kinds: Vec<String> = self.schemas
            .values()
            .filter_map(|n| n.kind.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        kinds.sort();
        kinds
    }
    
    /// Generate import statements for a schema
    pub fn imports_for(&self, id: &str, lang: &str) -> Vec<String> {
        let deps = self.closure(id, Direction::Outgoing, Some(1));
        let mut imports = Vec::new();
        
        // Helper to get type name from a node
        fn get_type_name(node: &SchemaNode) -> String {
            node.title.clone().unwrap_or_else(|| {
                node.path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .replace(".schema", "")
            })
        }
        
        // Add self
        if let Some(node) = self.get(id) {
            let name = get_type_name(node);
            let dir = node.path.parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");
            
            match lang {
                "rust" => {
                    imports.push(format!("use crate::{}::{};", dir.replace('/', "::"), name));
                }
                "typescript" | "ts" => {
                    imports.push(format!("import {{ {} }} from '@familiar/schemas/{}';", name, dir));
                }
                "python" | "py" => {
                    imports.push(format!("from familiar.schemas.{} import {}", dir.replace('/', "."), name));
                }
                _ => {}
            }
        }
        
        // Add deps
        for dep in deps {
            if let Some(node) = self.get(&dep.id) {
                let name = get_type_name(node);
                let dir = node.path.parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                
                match lang {
                    "rust" => {
                        imports.push(format!("use crate::{}::{};", dir.replace('/', "::"), name));
                    }
                    "typescript" | "ts" => {
                        imports.push(format!("import {{ {} }} from '@familiar/schemas/{}';", name, dir));
                    }
                    "python" | "py" => {
                        imports.push(format!("from familiar.schemas.{} import {}", dir.replace('/', "."), name));
                    }
                    _ => {}
                }
            }
        }
        
        imports.sort();
        imports.dedup();
        imports
    }
    
    /// Lint union schemas for ambiguity issues
    pub fn lint_unions(&self, id: &str) -> Vec<LintWarning> {
        let mut warnings = Vec::new();
        
        let Some(raw) = self.get_raw(id) else {
            return warnings;
        };
        
        // Check for untagged oneOf without discriminator
        if raw.get("oneOf").is_some() {
            let has_discriminator = raw.get("x-familiar-discriminator").is_some();
            let has_repr = raw.get("x-familiar-enum-repr").is_some();
            
            if !has_discriminator && !has_repr {
                warnings.push(LintWarning {
                    code: "UNTAGGED_UNION".to_string(),
                    message: "oneOf without x-familiar-discriminator or x-familiar-enum-repr may cause ambiguous parsing".to_string(),
                    severity: "warning".to_string(),
                });
            }
        }
        
        // Check for anyOf used where allOf might be intended
        if let Some(any_of) = raw.get("anyOf").and_then(|v| v.as_array()) {
            let all_objects = any_of.iter().all(|item| {
                item.get("type").map(|t| t == "object").unwrap_or(false)
                    || item.get("properties").is_some()
            });
            
            if all_objects {
                warnings.push(LintWarning {
                    code: "ANYOF_OBJECTS".to_string(),
                    message: "anyOf with all object types might be better as allOf (composition) or oneOf (union)".to_string(),
                    severity: "info".to_string(),
                });
            }
        }
        
        // Check for missing x-familiar-kind
        if raw.get("x-familiar-kind").is_none() {
            warnings.push(LintWarning {
                code: "MISSING_KIND".to_string(),
                message: "Schema is missing x-familiar-kind extension".to_string(),
                severity: "info".to_string(),
            });
        }
        
        warnings
    }
    
    /// Get all schemas
    pub fn all_schemas(&self) -> impl Iterator<Item = &SchemaNode> {
        self.schemas.values()
    }
    
    /// Get all schema IDs
    pub fn all_ids(&self) -> impl Iterator<Item = &SchemaId> {
        self.schemas.keys()
    }
    
    /// Load artifact indexes from JSON files
    /// 
    /// Artifact index files are generated by codegen and contain mappings from
    /// schema paths to generated type locations. Format: `generated.artifacts.json`
    pub fn load_artifact_indexes(&mut self, artifacts_dir: &Path) -> anyhow::Result<usize> {
        let mut loaded = 0;
        
        // Look for *.artifacts.json files
        for entry in WalkDir::new(artifacts_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if !path.to_string_lossy().ends_with(".artifacts.json") {
                continue;
            }
            
            let content = fs::read_to_string(path)?;
            let artifacts: Vec<serde_json::Value> = serde_json::from_str(&content)?;
            
            for artifact in artifacts {
                let schema_path = artifact.get("schema_path").and_then(|v| v.as_str());
                let lang = artifact.get("lang").and_then(|v| v.as_str());
                let file = artifact.get("file").and_then(|v| v.as_str());
                let line = artifact.get("line").and_then(|v| v.as_u64()).map(|l| l as u32);
                let type_name = artifact.get("type_name").and_then(|v| v.as_str());
                let type_kind = artifact.get("type_kind").and_then(|v| v.as_str());
                
                if let (Some(schema_path), Some(lang), Some(file), Some(line), Some(type_name), Some(type_kind)) 
                    = (schema_path, lang, file, line, type_name, type_kind) 
                {
                    // Try to find schema by path
                    let schema_id = self.by_path.get(&PathBuf::from(schema_path))
                        .cloned()
                        .or_else(|| self.resolve(schema_path).cloned());
                    
                    if let Some(id) = schema_id {
                        self.register_artifact(&id, GeneratedArtifact {
                            id: GeneratedArtifact::make_id(lang, type_name),
                            lang: lang.to_string(),
                            file: PathBuf::from(file),
                            line,
                            type_name: type_name.to_string(),
                            type_kind: type_kind.to_string(),
                        });
                        loaded += 1;
                    }
                }
            }
        }
        
        Ok(loaded)
    }
    
    /// Register a generated artifact for a schema
    /// Called by codegen after generating types
    /// Register a generated artifact for a schema
    /// 
    /// This populates multiple indexes for O(1) lookups:
    /// - schema_id → artifacts
    /// - artifact_id → schema_id
    /// - file → artifacts
    /// - lang → artifacts
    pub fn register_artifact(&mut self, schema_id: &str, mut artifact: GeneratedArtifact) {
        // Generate artifact ID if not set
        let artifact_id = artifact.artifact_id();
        artifact.id = artifact_id.clone();
        
        // Store in primary artifact map
        self.artifacts.insert(artifact_id.clone(), artifact.clone());
        
        // Index: schema → artifacts
        self.schema_to_artifacts
            .entry(schema_id.to_string())
            .or_default()
            .push(artifact_id.clone());
        
        // Index: artifact → schema (reverse lookup)
        self.artifact_to_schema.insert(artifact_id.clone(), schema_id.to_string());
        
        // Index: file → artifacts
        self.file_to_artifacts
            .entry(artifact.file.clone())
            .or_default()
            .push(artifact_id.clone());
        
        // Index: lang → artifacts
        self.lang_to_artifacts
            .entry(artifact.lang.clone())
            .or_default()
            .push(artifact_id.clone());
        
        // Also store in schema node for backward compatibility
        if let Some(node) = self.schemas.get_mut(schema_id) {
            let codegen = node.codegen.get_or_insert_with(CodegenMeta::default);
            if let Some(existing) = codegen.artifacts.iter_mut().find(|a| a.lang == artifact.lang) {
                *existing = artifact;
            } else {
                codegen.artifacts.push(artifact);
            }
        }
    }
    
    // ========== O(1) Artifact Lookups ==========
    
    /// Get all artifacts for a schema across all languages - O(1)
    pub fn get_artifacts(&self, schema_id: &str) -> Vec<&GeneratedArtifact> {
        self.schema_to_artifacts
            .get(schema_id)
            .map(|ids| ids.iter().filter_map(|id| self.artifacts.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get schema ID for an artifact - O(1)
    pub fn get_artifact_schema(&self, artifact_id: &str) -> Option<&str> {
        self.artifact_to_schema.get(artifact_id).map(|s| s.as_str())
    }
    
    /// Get all artifacts in a file - O(1)
    pub fn get_file_artifacts(&self, file: &Path) -> Vec<&GeneratedArtifact> {
        self.file_to_artifacts
            .get(file)
            .map(|ids| ids.iter().filter_map(|id| self.artifacts.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get all artifacts for a language - O(1)
    pub fn get_lang_artifacts(&self, lang: &str) -> Vec<&GeneratedArtifact> {
        self.lang_to_artifacts
            .get(lang)
            .map(|ids| ids.iter().filter_map(|id| self.artifacts.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get artifact by ID - O(1)
    pub fn get_artifact_by_id(&self, artifact_id: &str) -> Option<&GeneratedArtifact> {
        self.artifacts.get(artifact_id)
    }
    
    /// Get total artifact count
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }
    
    /// Get artifact for a specific language
    /// Get artifact for a schema in a specific language - O(1)
    pub fn get_artifact(&self, schema_id: &str, lang: &str) -> Option<&GeneratedArtifact> {
        let artifact_id = GeneratedArtifact::make_id(lang, 
            // Get type name from schema title or schema id
            self.schemas.get(schema_id)
                .and_then(|n| n.title.as_ref())
                .map(|t| t.as_str())
                .unwrap_or_else(|| schema_id.rsplit('/').next().unwrap_or(schema_id).trim_end_matches(".schema.json"))
        );
        self.artifacts.get(&artifact_id)
    }
    
    /// Get all schemas that have generated artifacts for a language - O(n) but indexed
    pub fn schemas_with_artifacts(&self, lang: &str) -> Vec<(&SchemaId, &GeneratedArtifact)> {
        self.lang_to_artifacts
            .get(lang)
            .map(|artifact_ids| {
                artifact_ids.iter()
                    .filter_map(|aid| {
                        let artifact = self.artifacts.get(aid)?;
                        let schema_id = self.artifact_to_schema.get(aid)?;
                        Some((schema_id, artifact))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get coverage stats: how many schemas have artifacts per language - O(1)
    pub fn artifact_coverage(&self) -> HashMap<String, (usize, usize)> {
        let total = self.schemas.len();
        
        // Use the lang_to_artifacts index for O(1) counts
        self.lang_to_artifacts
            .iter()
            .map(|(lang, artifacts)| (lang.clone(), (artifacts.len(), total)))
            .collect()
    }
    
    // ========== Graph-Based Artifact Queries ==========
    
    /// Find all artifacts that would be affected by changing a schema
    /// Uses graph traversal to find all dependent schemas, then their artifacts
    pub fn affected_artifacts(&self, schema_id: &str) -> Vec<&GeneratedArtifact> {
        // Get all schemas that depend on this one (transitive)
        let dependents = self.closure(schema_id, Direction::Incoming, None);
        
        // Collect artifacts for the schema itself + all dependents
        let mut affected = self.get_artifacts(schema_id);
        for dep in dependents {
            affected.extend(self.get_artifacts(&dep.id));
        }
        
        affected
    }
    
    /// Find all schemas needed to generate a specific artifact
    /// Uses graph traversal to find all dependencies
    pub fn artifact_dependencies(&self, artifact_id: &str) -> Vec<&SchemaNode> {
        let Some(schema_id) = self.artifact_to_schema.get(artifact_id) else {
            return Vec::new();
        };
        
        // Get all schemas this one depends on (transitive)
        let deps = self.closure(schema_id, Direction::Outgoing, None);
        
        let mut result = Vec::new();
        if let Some(node) = self.schemas.get(schema_id) {
            result.push(node);
        }
        for dep in deps {
            if let Some(node) = self.schemas.get(&dep.id) {
                result.push(node);
            }
        }
        
        result
    }
    
    /// Find co-located artifacts (same file) - useful for understanding file organization
    pub fn colocated_artifacts(&self, artifact_id: &str) -> Vec<&GeneratedArtifact> {
        let Some(artifact) = self.artifacts.get(artifact_id) else {
            return Vec::new();
        };
        
        self.get_file_artifacts(&artifact.file)
            .into_iter()
            .filter(|a| a.artifact_id() != artifact_id)
            .collect()
    }
}

/// Node in a closure result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureNode {
    pub id: SchemaId,
    pub depth: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub scc_boundary: bool,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: SchemaId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub path: PathBuf,
    pub score: i64,
}

/// Lint warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintWarning {
    pub code: String,
    pub message: String,
    pub severity: String,
}

