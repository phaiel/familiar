//! Schema Dependency Graph
//!
//! This module provides utilities for building and traversing a dependency graph
//! of JSON schemas based on their `$ref` relationships and `x-familiar-*` extensions.
//!
//! ## Key Patterns (from "Graph & Tree Traversals in Rust")
//!
//! - **Iterative DFS**: We use stack-based traversal (not recursive) to avoid
//!   stack overflow in `build.rs` where stack size may be limited.
//! - **Visited Set**: Essential to handle circular `$ref` relationships that are
//!   common in JSON schemas (e.g., User → Family → User).
//! - **Transitive Closure**: For feature-based filtering, we walk all reachable
//!   nodes from "feature roots" to include transitive dependencies.
//!
//! ## Edge Types
//!
//! The graph supports typed edges for different relationship types:
//! - **TypeRef**: Standard `$ref` dependency
//! - **RunsOn**: System → Node (x-familiar-service)
//! - **UsesQueue**: Node → Queue (x-familiar-queue reference)
//! - **Requires**: System → Component (x-familiar-depends)
//! - **Reads**: System → Entity (x-familiar-reads)
//! - **Writes**: System → Entity (x-familiar-writes)
//! - **ConnectsTo**: Component → Resource (x-familiar-resources)
//!
//! ## Usage
//!
//! ```ignore
//! use familiar_core::schemas::graph::{SchemaGraph, EdgeKind};
//!
//! let graph = SchemaGraph::from_directory(&schema_dir)?;
//!
//! // Get all schemas transitively required by a feature
//! let required = graph.transitive_deps(&["entities/Moment", "entities/Pulse"]);
//!
//! // Get blast radius - what would be affected if this resource fails?
//! let affected = graph.blast_radius("resources/postgres-main.resource.json");
//!
//! // Filter edges by type
//! let wire_edges = graph.edges_of_kind(&[EdgeKind::RunsOn, EdgeKind::UsesQueue]);
//!
//! // Export to DOT format for visualization
//! let dot = graph.to_dot();
//! ```

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Dfs, EdgeRef};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Types of edges in the schema graph.
/// 
/// Each edge type represents a different relationship discovered from
/// either standard `$ref` or `x-familiar-*` extensions, as well as
/// JSON Schema composition constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    // === Standard References ===
    /// Standard JSON Schema `$ref` dependency (cross-file)
    TypeRef,
    /// Local `$ref` within same file (#/definitions/X or #/$defs/X)
    LocalRef,
    
    // === Schema Composition ===
    /// allOf composition (inheritance/mixin)
    Extends,
    /// oneOf discriminated union variant
    VariantOf,
    /// anyOf union type option
    UnionOf,
    /// items array element type
    ItemType,
    /// additionalProperties map value type
    ValueType,
    /// properties.X object field type
    FieldType,
    
    // === x-familiar-* Extensions ===
    /// System executes on this Node (x-familiar-service)
    RunsOn,
    /// Node consumes from this Queue (x-familiar-queue as $ref)
    UsesQueue,
    /// System requires this Component (x-familiar-depends)
    Requires,
    /// System reads from this Entity (x-familiar-reads)
    Reads,
    /// System writes to this Entity (x-familiar-writes)
    Writes,
    /// Component connects to this Resource (x-familiar-resources)
    ConnectsTo,
    /// Input schema for a System (x-familiar-input)
    Input,
    /// Output schema for a System (x-familiar-output)
    Output,
}

impl EdgeKind {
    /// Get a short label for the edge type (for DOT visualization)
    pub fn label(&self) -> &'static str {
        match self {
            // Standard references
            EdgeKind::TypeRef => "ref",
            EdgeKind::LocalRef => "local",
            // Schema composition
            EdgeKind::Extends => "extends",
            EdgeKind::VariantOf => "variant",
            EdgeKind::UnionOf => "union",
            EdgeKind::ItemType => "item",
            EdgeKind::ValueType => "value",
            EdgeKind::FieldType => "field",
            // x-familiar-* extensions
            EdgeKind::RunsOn => "runs_on",
            EdgeKind::UsesQueue => "uses_queue",
            EdgeKind::Requires => "requires",
            EdgeKind::Reads => "reads",
            EdgeKind::Writes => "writes",
            EdgeKind::ConnectsTo => "connects_to",
            EdgeKind::Input => "input",
            EdgeKind::Output => "output",
        }
    }

    /// Get a color for the edge type (for DOT visualization)
    pub fn color(&self) -> &'static str {
        match self {
            // Standard references - grays
            EdgeKind::TypeRef => "#666666",       // Dark gray (cross-file ref)
            EdgeKind::LocalRef => "#AAAAAA",      // Light gray (local ref)
            // Schema composition - spectrum
            EdgeKind::Extends => "#4CAF50",       // Green (inheritance)
            EdgeKind::VariantOf => "#FF9800",     // Orange (oneOf variant)
            EdgeKind::UnionOf => "#FFC107",       // Amber (anyOf union)
            EdgeKind::ItemType => "#9C27B0",      // Purple (array item)
            EdgeKind::ValueType => "#E91E63",     // Pink (map value)
            EdgeKind::FieldType => "#9E9E9E",     // Gray (property field)
            // x-familiar-* extensions - infrastructure
            EdgeKind::RunsOn => "#2196F3",        // Blue (system -> node)
            EdgeKind::UsesQueue => "#673AB7",     // Deep Purple (queue)
            EdgeKind::Requires => "#FF5722",      // Deep Orange (component dep)
            EdgeKind::Reads => "#00BCD4",         // Cyan (read access)
            EdgeKind::Writes => "#F44336",        // Red (write access)
            EdgeKind::ConnectsTo => "#03A9F4",    // Light Blue (resource)
            EdgeKind::Input => "#8BC34A",         // Light Green (input)
            EdgeKind::Output => "#FF5722",        // Deep Orange (output)
        }
    }

    /// Check if this edge type represents a schema composition construct
    pub fn is_composition(&self) -> bool {
        matches!(self, 
            EdgeKind::Extends | 
            EdgeKind::VariantOf | 
            EdgeKind::UnionOf | 
            EdgeKind::ItemType | 
            EdgeKind::ValueType | 
            EdgeKind::FieldType
        )
    }

    /// Check if this edge type represents an infrastructure relationship
    pub fn is_infrastructure(&self) -> bool {
        matches!(self,
            EdgeKind::RunsOn |
            EdgeKind::UsesQueue |
            EdgeKind::Requires |
            EdgeKind::Reads |
            EdgeKind::Writes |
            EdgeKind::ConnectsTo |
            EdgeKind::Input |
            EdgeKind::Output
        )
    }
}

/// Metadata about a schema node in the graph.
/// 
/// Nodes can represent either:
/// - A schema file (e.g., "entities/Moment.schema.json")
/// - A local definition within a file (e.g., "entities/Moment.schema.json#LoginStatus")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaNode {
    /// Schema identifier (relative path like "entities/Moment.schema.json" 
    /// or "entities/Moment.schema.json#LoginStatus" for local definitions)
    pub id: String,
    /// File path containing this schema (same as id for root schemas)
    pub file_path: String,
    /// Local definition name if this is a #/definitions/X or #/$defs/X node
    pub definition: Option<String>,
    /// Schema kind from x-familiar-kind extension
    pub kind: Option<String>,
    /// Schema title
    pub title: Option<String>,
}

/// Information about an orphan schema (no incoming edges in the graph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanInfo {
    /// Schema identifier (e.g., "auth/CreateUserInput.schema.json")
    pub schema_id: String,
    /// File path to the schema
    pub file_path: String,
    /// Category/directory (e.g., "auth", "tools")
    pub category: String,
    /// Schema kind from x-familiar-kind extension
    pub kind: Option<String>,
    /// Whether this orphan is an expected root node (ecs/, queues/, nodes/, systems/)
    pub is_expected_root: bool,
    /// Whether this schema has outgoing edges (references other schemas)
    /// If true, this is a "consumer" not a true orphan
    pub has_outgoing: bool,
}

/// A dependency graph of JSON schemas.
///
/// Nodes are schema identifiers (relative paths like "entities/Moment.schema.json").
/// Edges represent typed relationships discovered from `$ref` and `x-familiar-*` extensions.
#[derive(Debug)]
pub struct SchemaGraph {
    /// The underlying petgraph directed graph with typed edges
    graph: DiGraph<SchemaNode, EdgeKind>,
    /// Map from schema identifier to node index
    node_map: HashMap<String, NodeIndex>,
}

impl SchemaGraph {
    /// Create a new empty schema graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Build a schema graph from a directory of JSON schema files.
    ///
    /// This performs a "Discovery Walk" as described in the traversal article:
    /// 1. **Seed**: Start with all `.json` files in the directory
    /// 2. **Visit**: Parse each file for `$ref` keys and `x-familiar-*` extensions
    /// 3. **Edge**: Add typed edges based on the relationship type
    ///
    /// # Arguments
    /// * `schema_dir` - Path to the json-schema directory
    ///
    /// # Returns
    /// A `SchemaGraph` with all schemas as nodes and typed edges.
    pub fn from_directory(schema_dir: &Path) -> Result<Self, std::io::Error> {
        Self::from_directory_with_depth(schema_dir, 0)
    }

    /// Build a schema graph from a directory with configurable depth.
    ///
    /// # Arguments
    /// * `schema_dir` - Path to the json-schema directory
    /// * `depth` - How deep to traverse into properties (0 = unlimited)
    ///
    /// # Returns
    /// A `SchemaGraph` with all schemas as nodes and typed edges.
    pub fn from_directory_with_depth(schema_dir: &Path, depth: usize) -> Result<Self, std::io::Error> {
        let mut graph = Self::new();
        
        // First pass: discover all schema files and create nodes with metadata
        let schema_files = discover_schema_files(schema_dir)?;
        
        // Cache parsed JSON to avoid re-parsing
        let mut parsed_schemas: HashMap<PathBuf, serde_json::Value> = HashMap::new();
        
        for path in &schema_files {
            let relative = path
                .strip_prefix(schema_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().to_string());
            
            // Parse schema to extract metadata
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let kind = json.get("x-familiar-kind")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let title = json.get("title")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    graph.add_node_with_metadata(&relative, kind, title);
                    
                    // Also create nodes for local definitions
                    Self::add_definition_nodes(&mut graph, &json, &relative);
                    
                    // Cache for second pass
                    parsed_schemas.insert(path.clone(), json);
                }
            }
        }
        
        // Second pass: add typed edges
        for (path, json) in &parsed_schemas {
            let relative = path
                .strip_prefix(schema_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().to_string());
            
            // 1. Extract edges from x-familiar-* extensions (infrastructure)
            let familiar_refs = extract_typed_refs(json);
            for (ref_path, edge_kind) in familiar_refs {
                let normalized = normalize_ref(&relative, &ref_path);
                if !normalized.is_empty() && graph.node_map.contains_key(&normalized) {
                    graph.add_typed_edge(&relative, &normalized, edge_kind);
                }
            }
            
            // 2. Extract edges from schema composition (allOf, oneOf, anyOf, etc.)
            let composition_refs = extract_all_type_refs(json, &relative, depth, 0);
            for (target, edge_kind) in composition_refs {
                // Check if target exists in graph (it might be a local def or external file)
                if graph.node_map.contains_key(&target) {
                    // Avoid duplicate edges
                    if !graph.has_edge(&relative, &target) {
                        graph.add_typed_edge(&relative, &target, edge_kind);
                    }
                }
            }
            
            // 3. Add edges FROM local definitions to their references
            Self::add_definition_edges(&mut graph, json, &relative, depth);
        }
        
        Ok(graph)
    }

    /// Add nodes for all definitions/$defs in a schema
    fn add_definition_nodes(graph: &mut SchemaGraph, json: &serde_json::Value, file_path: &str) {
        // Check for "definitions" (JSON Schema draft-04 to draft-07)
        if let Some(serde_json::Value::Object(defs)) = json.get("definitions") {
            for (def_name, def_schema) in defs {
                let title = def_schema.get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let kind = def_schema.get("x-familiar-kind")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| Some("definition".to_string()));
                
                graph.add_definition_node(file_path, def_name, kind, title);
            }
        }
        
        // Check for "$defs" (JSON Schema draft 2019-09+)
        if let Some(serde_json::Value::Object(defs)) = json.get("$defs") {
            for (def_name, def_schema) in defs {
                let title = def_schema.get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let kind = def_schema.get("x-familiar-kind")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| Some("definition".to_string()));
                
                graph.add_definition_node(file_path, def_name, kind, title);
            }
        }
    }

    /// Add edges from local definitions to their references
    fn add_definition_edges(graph: &mut SchemaGraph, json: &serde_json::Value, file_path: &str, depth: usize) {
        // Process definitions
        if let Some(serde_json::Value::Object(defs)) = json.get("definitions") {
            for (def_name, def_schema) in defs {
                let def_id = format!("{}#{}", file_path, def_name);
                let refs = extract_all_type_refs(def_schema, file_path, depth, 0);
                
                for (target, edge_kind) in refs {
                    if graph.node_map.contains_key(&target) && graph.node_map.contains_key(&def_id) {
                        if !graph.has_edge(&def_id, &target) {
                            graph.add_typed_edge(&def_id, &target, edge_kind);
                        }
                    }
                }
            }
        }
        
        // Process $defs
        if let Some(serde_json::Value::Object(defs)) = json.get("$defs") {
            for (def_name, def_schema) in defs {
                let def_id = format!("{}#{}", file_path, def_name);
                let refs = extract_all_type_refs(def_schema, file_path, depth, 0);
                
                for (target, edge_kind) in refs {
                    if graph.node_map.contains_key(&target) && graph.node_map.contains_key(&def_id) {
                        if !graph.has_edge(&def_id, &target) {
                            graph.add_typed_edge(&def_id, &target, edge_kind);
                        }
                    }
                }
            }
        }
    }

    /// Add a node to the graph if it doesn't exist.
    pub fn add_node(&mut self, id: &str) -> NodeIndex {
        self.add_node_with_metadata(id, None, None)
    }

    /// Add a node with metadata to the graph if it doesn't exist.
    pub fn add_node_with_metadata(&mut self, id: &str, kind: Option<String>, title: Option<String>) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(id) {
            idx
        } else {
            // Parse id to extract file_path and definition
            let (file_path, definition) = if let Some(hash_pos) = id.find('#') {
                (id[..hash_pos].to_string(), Some(id[hash_pos + 1..].to_string()))
            } else {
                (id.to_string(), None)
            };
            
            let node = SchemaNode {
                id: id.to_string(),
                file_path,
                definition,
                kind,
                title,
            };
            let idx = self.graph.add_node(node);
            self.node_map.insert(id.to_string(), idx);
            idx
        }
    }

    /// Add a local definition node to the graph.
    pub fn add_definition_node(&mut self, file_path: &str, def_name: &str, kind: Option<String>, title: Option<String>) -> NodeIndex {
        let id = format!("{}#{}", file_path, def_name);
        if let Some(&idx) = self.node_map.get(&id) {
            idx
        } else {
            let node = SchemaNode {
                id: id.clone(),
                file_path: file_path.to_string(),
                definition: Some(def_name.to_string()),
                kind,
                title,
            };
            let idx = self.graph.add_node(node);
            self.node_map.insert(id, idx);
            idx
        }
    }

    /// Add an edge between two schemas (legacy method, uses TypeRef).
    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.add_typed_edge(from, to, EdgeKind::TypeRef);
    }

    /// Add a typed edge between two schemas.
    pub fn add_typed_edge(&mut self, from: &str, to: &str, kind: EdgeKind) {
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);
        
        // Add edge (allow multiple edges with different types)
        self.graph.add_edge(from_idx, to_idx, kind);
    }

    /// Check if an edge exists between two schemas.
    pub fn has_edge(&self, from: &str, to: &str) -> bool {
        if let (Some(&from_idx), Some(&to_idx)) = (self.node_map.get(from), self.node_map.get(to)) {
            self.graph.contains_edge(from_idx, to_idx)
        } else {
            false
        }
    }

    /// Get the kind of a schema node.
    pub fn kind(&self, schema_id: &str) -> Option<&str> {
        self.node_map.get(schema_id)
            .and_then(|&idx| self.graph[idx].kind.as_deref())
    }

    /// Get all transitive dependencies for a set of root schemas.
    ///
    /// This uses an **iterative DFS** (stack-based) approach as recommended
    /// in the traversal article. This is safer than recursive DFS because:
    /// - It won't stack overflow on deeply nested schemas
    /// - It works reliably in `build.rs` where stack size may be limited
    ///
    /// # Arguments
    /// * `roots` - Schema identifiers to start from (e.g., ["entities/Moment"])
    ///
    /// # Returns
    /// A `HashSet` of all schema identifiers that are transitively required.
    pub fn transitive_deps(&self, roots: &[&str]) -> HashSet<String> {
        let mut required: HashSet<String> = HashSet::new();
        let mut stack: Vec<NodeIndex> = Vec::new();
        
        // 1. Add all root schemas to the stack
        for root in roots {
            if let Some(&idx) = self.node_map.get(*root) {
                stack.push(idx);
            }
        }
        
        // 2. Iterative DFS walk (stack-based, not recursive!)
        // This is the pattern from the article that avoids stack overflow
        while let Some(node_idx) = stack.pop() {
            let schema_id = &self.graph[node_idx].id;
            
            // Skip if already visited (handles circular refs)
            if !required.insert(schema_id.clone()) {
                continue;
            }
            
            // Add all dependencies (outgoing edges) to the stack
            for neighbor in self.graph.neighbors_directed(node_idx, Direction::Outgoing) {
                stack.push(neighbor);
            }
        }
        
        required
    }

    /// Get all transitive dependencies, optionally filtered by edge types.
    pub fn transitive_deps_filtered(&self, roots: &[&str], edge_kinds: &[EdgeKind]) -> HashSet<String> {
        let mut required: HashSet<String> = HashSet::new();
        let mut stack: Vec<NodeIndex> = Vec::new();
        let edge_filter: HashSet<_> = edge_kinds.iter().collect();
        
        for root in roots {
            if let Some(&idx) = self.node_map.get(*root) {
                stack.push(idx);
            }
        }
        
        while let Some(node_idx) = stack.pop() {
            let schema_id = &self.graph[node_idx].id;
            
            if !required.insert(schema_id.clone()) {
                continue;
            }
            
            // Only follow edges of the specified types
            for edge in self.graph.edges_directed(node_idx, Direction::Outgoing) {
                if edge_filter.is_empty() || edge_filter.contains(edge.weight()) {
                    stack.push(edge.target());
                }
            }
        }
        
        required
    }

    /// Get all transitive dependencies using petgraph's built-in Dfs.
    ///
    /// This is an alternative to `transitive_deps` that uses petgraph's
    /// optimized DFS implementation.
    pub fn transitive_deps_petgraph(&self, roots: &[&str]) -> HashSet<String> {
        let mut required: HashSet<String> = HashSet::new();
        
        for root in roots {
            if let Some(&start_idx) = self.node_map.get(*root) {
                // petgraph::visit::Dfs handles the visited set internally
                let mut dfs = Dfs::new(&self.graph, start_idx);
                
                while let Some(node_idx) = dfs.next(&self.graph) {
                    required.insert(self.graph[node_idx].id.clone());
                }
            }
        }
        
        required
    }

    /// Find all nodes affected if this node fails (reverse reachability).
    ///
    /// This is the "Blast Radius" analysis - if a resource goes down,
    /// which systems are affected?
    ///
    /// # Arguments
    /// * `node_id` - The schema that might fail (e.g., a resource)
    /// * `edge_kinds` - Types of edges to follow in reverse (empty = all)
    ///
    /// # Returns
    /// All schemas that would be affected (reverse transitive closure)
    pub fn blast_radius(&self, node_id: &str, edge_kinds: &[EdgeKind]) -> HashSet<String> {
        self.reverse_transitive_deps(node_id, edge_kinds)
    }

    /// Get all schemas that transitively depend on this one (reverse DFS).
    pub fn reverse_transitive_deps(&self, node_id: &str, edge_kinds: &[EdgeKind]) -> HashSet<String> {
        let mut affected: HashSet<String> = HashSet::new();
        let mut stack: Vec<NodeIndex> = Vec::new();
        let edge_filter: HashSet<_> = edge_kinds.iter().collect();
        
        if let Some(&start_idx) = self.node_map.get(node_id) {
            stack.push(start_idx);
        }
        
        while let Some(node_idx) = stack.pop() {
            let schema_id = &self.graph[node_idx].id;
            
            if !affected.insert(schema_id.clone()) {
                continue;
            }
            
            // Follow incoming edges (reverse direction)
            for edge in self.graph.edges_directed(node_idx, Direction::Incoming) {
                if edge_filter.is_empty() || edge_filter.contains(edge.weight()) {
                    stack.push(edge.source());
                }
            }
        }
        
        affected
    }

    /// Get direct dependencies (schemas this one references).
    pub fn dependencies(&self, schema_id: &str) -> Vec<&str> {
        self.node_map
            .get(schema_id)
            .map(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .map(|n| self.graph[n].id.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get direct dependents (schemas that reference this one).
    pub fn dependents(&self, schema_id: &str) -> Vec<&str> {
        self.node_map
            .get(schema_id)
            .map(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .map(|n| self.graph[n].id.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all schema identifiers in the graph.
    pub fn all_schemas(&self) -> Vec<&str> {
        self.graph.node_weights().map(|s| s.id.as_str()).collect()
    }

    /// Get the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get edges filtered by type.
    pub fn edges_of_kind(&self, kinds: &[EdgeKind]) -> Vec<(&str, &str, EdgeKind)> {
        let kind_set: HashSet<_> = kinds.iter().collect();
        
        self.graph
            .edge_references()
            .filter(|e| kind_set.is_empty() || kind_set.contains(e.weight()))
            .map(|e| {
                let from = self.graph[e.source()].id.as_str();
                let to = self.graph[e.target()].id.as_str();
                (from, to, *e.weight())
            })
            .collect()
    }

    /// Count edges by type.
    pub fn edge_type_counts(&self) -> HashMap<EdgeKind, usize> {
        let mut counts: HashMap<EdgeKind, usize> = HashMap::new();
        
        for edge in self.graph.edge_references() {
            *counts.entry(*edge.weight()).or_insert(0) += 1;
        }
        
        counts
    }

    /// Get schemas sorted by dependency count (most dependencies first).
    pub fn by_dependency_count(&self) -> Vec<(&str, usize)> {
        let mut result: Vec<_> = self.graph
            .node_indices()
            .map(|idx| {
                let name = self.graph[idx].id.as_str();
                let deps = self.graph.neighbors_directed(idx, Direction::Outgoing).count();
                (name, deps)
            })
            .collect();
        
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Get schemas sorted by reference count (most referenced first).
    pub fn by_reference_count(&self) -> Vec<(&str, usize)> {
        let mut result: Vec<_> = self.graph
            .node_indices()
            .map(|idx| {
                let name = self.graph[idx].id.as_str();
                let refs = self.graph.neighbors_directed(idx, Direction::Incoming).count();
                (name, refs)
            })
            .collect();
        
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Get all orphan schemas (nodes with no incoming edges).
    /// 
    /// Orphan schemas are schemas that are not referenced by any other schema.
    /// This excludes local definitions (schemas with # in their ID).
    /// 
    /// # Returns
    /// A vector of tuples: (schema_id, file_path, kind, is_expected_root)
    /// where is_expected_root indicates if the schema is in a category that's
    /// typically a root (ecs/, queues/, nodes/, systems/).
    /// has_outgoing indicates if the schema references other schemas (is a consumer).
    pub fn orphan_schemas(&self) -> Vec<OrphanInfo> {
        let expected_root_categories = ["ecs", "queues", "nodes", "systems", "resources"];
        
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph[idx];
                
                // Skip local definitions (they have # in the ID)
                if node.id.contains('#') {
                    return None;
                }
                
                // Check if has incoming edges
                let incoming_count = self.graph.neighbors_directed(idx, Direction::Incoming).count();
                if incoming_count > 0 {
                    return None;
                }
                
                // Check if has outgoing edges (is a consumer)
                let outgoing_count = self.graph.neighbors_directed(idx, Direction::Outgoing).count();
                
                // Extract category from path
                let category = node.file_path
                    .split('/')
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                
                // Check if it's an expected root category
                let is_expected_root = expected_root_categories.iter()
                    .any(|c| category == *c);
                
                Some(OrphanInfo {
                    schema_id: node.id.clone(),
                    file_path: node.file_path.clone(),
                    category,
                    kind: node.kind.clone(),
                    is_expected_root,
                    has_outgoing: outgoing_count > 0,
                })
            })
            .collect()
    }
    
    /// Get truly isolated schemas - no incoming AND no outgoing edges.
    /// These are the real orphans that need investigation.
    pub fn truly_isolated_schemas(&self) -> Vec<OrphanInfo> {
        self.orphan_schemas()
            .into_iter()
            .filter(|o| !o.has_outgoing)
            .collect()
    }
    
    /// Get consumer-only schemas - have outgoing edges but no incoming.
    /// These are "leaf" schemas that consume primitives/types but aren't consumed.
    pub fn consumer_only_schemas(&self) -> Vec<OrphanInfo> {
        self.orphan_schemas()
            .into_iter()
            .filter(|o| o.has_outgoing)
            .collect()
    }

    /// Get orphan schemas grouped by category.
    pub fn orphans_by_category(&self) -> HashMap<String, Vec<OrphanInfo>> {
        let orphans = self.orphan_schemas();
        let mut by_category: HashMap<String, Vec<OrphanInfo>> = HashMap::new();
        
        for orphan in orphans {
            by_category
                .entry(orphan.category.clone())
                .or_default()
                .push(orphan);
        }
        
        by_category
    }

    /// Export the graph in DOT format for visualization with Graphviz.
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph G {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n");
        
        // Add nodes
        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            let label = node.title.as_deref().unwrap_or(&node.id);
            let color = match node.kind.as_deref() {
                Some("node") => "#2196F3",
                Some("system") => "#4CAF50",
                Some("resource") => "#FF9800",
                Some("queue") => "#9C27B0",
                Some("entity") => "#00BCD4",
                Some("primitive") => "#607D8B",
                _ => "#9E9E9E",
            };
            dot.push_str(&format!("  \"{}\" [label=\"{}\", fillcolor=\"{}\", style=filled];\n", 
                node.id, label, color));
        }
        
        // Add edges with colors
        for edge in self.graph.edge_references() {
            let from = &self.graph[edge.source()].id;
            let to = &self.graph[edge.target()].id;
            let color = edge.weight().color();
            let label = edge.weight().label();
            dot.push_str(&format!("  \"{}\" -> \"{}\" [color=\"{}\", label=\"{}\"];\n", 
                from, to, color, label));
        }
        
        dot.push_str("}\n");
        dot
    }

    /// Export the graph in DOT format filtered by edge types.
    pub fn to_dot_filtered(&self, edge_kinds: &[EdgeKind]) -> String {
        let kind_set: HashSet<_> = edge_kinds.iter().collect();
        let mut dot = String::from("digraph G {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n");
        
        // Collect nodes that have matching edges
        let mut relevant_nodes: HashSet<NodeIndex> = HashSet::new();
        for edge in self.graph.edge_references() {
            if kind_set.is_empty() || kind_set.contains(edge.weight()) {
                relevant_nodes.insert(edge.source());
                relevant_nodes.insert(edge.target());
            }
        }
        
        // Add only relevant nodes
        for idx in relevant_nodes.iter() {
            let node = &self.graph[*idx];
            let label = node.title.as_deref().unwrap_or(&node.id);
            dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", node.id, label));
        }
        
        // Add filtered edges
        for edge in self.graph.edge_references() {
            if kind_set.is_empty() || kind_set.contains(edge.weight()) {
                let from = &self.graph[edge.source()].id;
                let to = &self.graph[edge.target()].id;
                let color = edge.weight().color();
                dot.push_str(&format!("  \"{}\" -> \"{}\" [color=\"{}\"];\n", from, to, color));
            }
        }
        
        dot.push_str("}\n");
        dot
    }

    /// Perform topological sort for codegen ordering.
    ///
    /// Returns schemas in an order where dependencies come before dependents.
    /// This is useful for generating a single file where types are defined
    /// in the correct order.
    ///
    /// Returns `None` if there are cycles in the graph.
    pub fn topological_order(&self) -> Option<Vec<&str>> {
        petgraph::algo::toposort(&self.graph, None)
            .ok()
            .map(|sorted| {
                sorted
                    .into_iter()
                    .map(|idx| self.graph[idx].id.as_str())
                    .collect()
            })
    }
}

impl Default for SchemaGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover all JSON schema files in a directory (recursively).
fn discover_schema_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    
    if !dir.exists() {
        return Ok(files);
    }
    
    // Use iterative approach (stack-based) instead of recursive
    let mut stack = vec![dir.to_path_buf()];
    
    while let Some(current) = stack.pop() {
        if current.is_dir() {
            for entry in fs::read_dir(&current)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().map_or(false, |e| e == "json") {
                    files.push(path);
                }
            }
        }
    }
    
    files.sort();
    Ok(files)
}

/// Extract all `$ref` values from a JSON schema string.
///
/// Uses iterative traversal of the JSON structure to avoid stack overflow
/// on deeply nested schemas.
fn extract_refs(content: &str) -> Vec<String> {
    let mut refs = Vec::new();
    
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        // Iterative traversal using a stack (not recursive!)
        let mut stack = vec![&json];
        
        while let Some(value) = stack.pop() {
            match value {
                serde_json::Value::Object(map) => {
                    // Check for $ref key
                    if let Some(serde_json::Value::String(r)) = map.get("$ref") {
                        refs.push(r.clone());
                    }
                    // Add all values to stack for further processing
                    for v in map.values() {
                        stack.push(v);
                    }
                }
                serde_json::Value::Array(arr) => {
                    for v in arr {
                        stack.push(v);
                    }
                }
                _ => {}
            }
        }
    }
    
    refs
}

/// Extract ALL type references from a JSON schema, including composition constructs.
/// 
/// This function traverses the entire schema and extracts typed edges for:
/// - `$ref` (both local #/definitions/X and external file refs)
/// - `allOf` (inheritance/composition)
/// - `oneOf` (discriminated unions)
/// - `anyOf` (union types)
/// - `items` (array element types)
/// - `additionalProperties` (map value types)
/// - `properties` (object field types)
///
/// The `base_path` is the file path of the schema being parsed, used to resolve
/// relative `$ref` paths.
///
/// The `depth` parameter controls how deep into properties to traverse:
/// - 0 = unlimited depth
/// - n = stop after n levels of property traversal
fn extract_all_type_refs(
    schema: &serde_json::Value, 
    base_path: &str,
    depth: usize,
    current_depth: usize,
) -> Vec<(String, EdgeKind)> {
    let mut refs = Vec::new();
    
    // Check depth limit for property traversal
    let at_depth_limit = depth > 0 && current_depth >= depth;
    
    if let serde_json::Value::Object(map) = schema {
        // 1. $ref (local or external)
        if let Some(serde_json::Value::String(ref_val)) = map.get("$ref") {
            if ref_val.starts_with("#/definitions/") || ref_val.starts_with("#/$defs/") {
                // Local: #/definitions/Foo -> "file#Foo"
                let def_name = ref_val.split('/').last().unwrap_or("");
                refs.push((format!("{}#{}", base_path, def_name), EdgeKind::LocalRef));
            } else if !ref_val.starts_with('#') {
                // External: ../primitives/UUID.json
                let normalized = normalize_ref(base_path, ref_val);
                if !normalized.is_empty() {
                    refs.push((normalized, EdgeKind::TypeRef));
                }
            }
            // Skip other keys in a $ref object (JSON Schema says $ref should be alone)
            return refs;
        }
        
        // 2. allOf -> Extends edges
        if let Some(serde_json::Value::Array(all_of)) = map.get("allOf") {
            for item in all_of {
                let item_refs = extract_all_type_refs(item, base_path, depth, current_depth);
                for (target, kind) in item_refs {
                    // Promote TypeRef/LocalRef to Extends for allOf items
                    let edge_kind = if kind == EdgeKind::TypeRef || kind == EdgeKind::LocalRef {
                        EdgeKind::Extends
                    } else {
                        kind
                    };
                    refs.push((target, edge_kind));
                }
            }
        }
        
        // 3. oneOf -> VariantOf edges
        if let Some(serde_json::Value::Array(one_of)) = map.get("oneOf") {
            for item in one_of {
                let item_refs = extract_all_type_refs(item, base_path, depth, current_depth);
                for (target, kind) in item_refs {
                    let edge_kind = if kind == EdgeKind::TypeRef || kind == EdgeKind::LocalRef {
                        EdgeKind::VariantOf
                    } else {
                        kind
                    };
                    refs.push((target, edge_kind));
                }
            }
        }
        
        // 4. anyOf -> UnionOf edges
        if let Some(serde_json::Value::Array(any_of)) = map.get("anyOf") {
            for item in any_of {
                let item_refs = extract_all_type_refs(item, base_path, depth, current_depth);
                for (target, kind) in item_refs {
                    let edge_kind = if kind == EdgeKind::TypeRef || kind == EdgeKind::LocalRef {
                        EdgeKind::UnionOf
                    } else {
                        kind
                    };
                    refs.push((target, edge_kind));
                }
            }
        }
        
        // 5. items -> ItemType edge
        if let Some(items) = map.get("items") {
            let item_refs = extract_all_type_refs(items, base_path, depth, current_depth);
            for (target, kind) in item_refs {
                let edge_kind = if kind == EdgeKind::TypeRef || kind == EdgeKind::LocalRef {
                    EdgeKind::ItemType
                } else {
                    kind
                };
                refs.push((target, edge_kind));
            }
        }
        
        // 6. additionalProperties -> ValueType edge (if it's a schema, not boolean)
        if let Some(add_props) = map.get("additionalProperties") {
            if add_props.is_object() {
                let prop_refs = extract_all_type_refs(add_props, base_path, depth, current_depth);
                for (target, kind) in prop_refs {
                    let edge_kind = if kind == EdgeKind::TypeRef || kind == EdgeKind::LocalRef {
                        EdgeKind::ValueType
                    } else {
                        kind
                    };
                    refs.push((target, edge_kind));
                }
            }
        }
        
        // 7. properties -> FieldType edges (depth-limited)
        if !at_depth_limit {
            if let Some(serde_json::Value::Object(props)) = map.get("properties") {
                for (_field_name, field_schema) in props {
                    let field_refs = extract_all_type_refs(field_schema, base_path, depth, current_depth + 1);
                    for (target, kind) in field_refs {
                        // Keep existing kind for nested refs, but use FieldType for direct refs
                        let edge_kind = if kind == EdgeKind::TypeRef || kind == EdgeKind::LocalRef {
                            EdgeKind::FieldType
                        } else {
                            kind
                        };
                        refs.push((target, edge_kind));
                    }
                }
            }
        }
        
        // 8. Recurse into nested schemas (definitions, patternProperties, etc.)
        // But don't traverse into x-familiar-* as those are handled separately
        for (key, value) in map {
            if key.starts_with("x-familiar") {
                continue; // Skip x-familiar-* extensions (handled by extract_typed_refs)
            }
            if key == "definitions" || key == "$defs" {
                // Skip definitions here - they're added as separate nodes
                continue;
            }
            if key == "properties" || key == "items" || key == "allOf" || 
               key == "oneOf" || key == "anyOf" || key == "additionalProperties" {
                // Already handled above
                continue;
            }
            // For other nested schemas (e.g., "not", "if", "then", "else")
            if value.is_object() && !at_depth_limit {
                refs.extend(extract_all_type_refs(value, base_path, depth, current_depth));
            }
        }
    }
    
    refs
}

/// Extract typed references from `x-familiar-*` extensions.
///
/// This parses the schema and finds `$ref` values inside specific extension keys,
/// mapping them to appropriate edge types.
fn extract_typed_refs(json: &serde_json::Value) -> Vec<(String, EdgeKind)> {
    let mut refs: Vec<(String, EdgeKind)> = Vec::new();
    
    if let serde_json::Value::Object(map) = json {
        // x-familiar-service: { "$ref": "..." } -> RunsOn
        if let Some(service) = map.get("x-familiar-service") {
            if let Some(ref_val) = extract_ref_from_value(service) {
                refs.push((ref_val, EdgeKind::RunsOn));
            }
        }
        
        // x-familiar-depends: [{ "$ref": "..." }, ...] -> Requires
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-depends") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::Requires));
                }
            }
        }
        
        // x-familiar-resources: [{ "$ref": "..." }, ...] -> ConnectsTo
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-resources") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::ConnectsTo));
                }
            }
        }
        
        // x-familiar-reads: [{ "$ref": "..." }, ...] -> Reads
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-reads") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::Reads));
                }
            }
        }
        
        // x-familiar-writes: [{ "$ref": "..." }, ...] -> Writes
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-writes") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::Writes));
                }
            }
        }
        
        // x-familiar-input: { "$ref": "..." } -> Input
        if let Some(input) = map.get("x-familiar-input") {
            if let Some(ref_val) = extract_ref_from_value(input) {
                refs.push((ref_val, EdgeKind::Input));
            }
        }
        
        // x-familiar-output: { "$ref": "..." } -> Output
        if let Some(output) = map.get("x-familiar-output") {
            if let Some(ref_val) = extract_ref_from_value(output) {
                refs.push((ref_val, EdgeKind::Output));
            }
        }
        
        // x-familiar-systems: [{ "$ref": "..." }, ...] -> TypeRef (for nodes)
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-systems") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::TypeRef));
                }
            }
        }
        
        // x-familiar-components: [{ "$ref": "..." }, ...] -> Requires (for nodes)
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-components") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::Requires));
                }
            }
        }
        
        // x-familiar-consumers: [{ "$ref": "..." }, ...] -> UsesQueue (reverse - for queues)
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-consumers") {
            for item in arr {
                if let Some(ref_val) = extract_ref_from_value(item) {
                    refs.push((ref_val, EdgeKind::UsesQueue));
                }
            }
        }
        
        // x-familiar-config: { "$ref": "..." } -> TypeRef
        if let Some(config) = map.get("x-familiar-config") {
            if let Some(ref_val) = extract_ref_from_value(config) {
                refs.push((ref_val, EdgeKind::TypeRef));
            }
        }
    }
    
    refs
}

/// Extract a $ref value from a JSON value (handles { "$ref": "..." } pattern)
fn extract_ref_from_value(value: &serde_json::Value) -> Option<String> {
    if let serde_json::Value::Object(map) = value {
        if let Some(serde_json::Value::String(ref_val)) = map.get("$ref") {
            return Some(ref_val.clone());
        }
    }
    None
}

/// Normalize a `$ref` path relative to the current schema.
///
/// Handles different `$ref` formats:
/// - `"primitives/Timestamp.schema.json"` (relative)
/// - `"../primitives/Timestamp.schema.json"` (parent relative)
/// - `"#/definitions/Foo"` (local - ignored)
fn normalize_ref(current_path: &str, ref_path: &str) -> String {
    // Skip local references
    if ref_path.starts_with('#') {
        return String::new();
    }
    
    if ref_path.starts_with("../") {
        // Parent-relative path - resolve against current directory
        let current_dir = Path::new(current_path).parent().unwrap_or(Path::new(""));
        let resolved = current_dir.join(ref_path);
        let resolved_str = resolved.to_string_lossy().to_string();
        
        // Normalize the path (remove .. components)
        let mut components: Vec<&str> = Vec::new();
        for part in resolved_str.split('/') {
            match part {
                ".." => { components.pop(); }
                "." | "" => {}
                _ => components.push(part),
            }
        }
        components.join("/")
    } else {
        // Absolute or simple relative path
        ref_path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = SchemaGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_nodes_and_edges() {
        let mut graph = SchemaGraph::new();
        graph.add_node("entities/Moment.schema.json");
        graph.add_node("primitives/Timestamp.schema.json");
        graph.add_edge("entities/Moment.schema.json", "primitives/Timestamp.schema.json");
        
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_typed_edges() {
        let mut graph = SchemaGraph::new();
        graph.add_node("systems/FatesGate.system.json");
        graph.add_node("nodes/daemon.node.json");
        graph.add_node("entities/Moment.schema.json");
        
        graph.add_typed_edge("systems/FatesGate.system.json", "nodes/daemon.node.json", EdgeKind::RunsOn);
        graph.add_typed_edge("systems/FatesGate.system.json", "entities/Moment.schema.json", EdgeKind::Reads);
        
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        
        let counts = graph.edge_type_counts();
        assert_eq!(counts.get(&EdgeKind::RunsOn), Some(&1));
        assert_eq!(counts.get(&EdgeKind::Reads), Some(&1));
    }

    #[test]
    fn test_transitive_deps() {
        let mut graph = SchemaGraph::new();
        
        // A -> B -> C
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        
        let deps = graph.transitive_deps(&["A"]);
        assert!(deps.contains("A"));
        assert!(deps.contains("B"));
        assert!(deps.contains("C"));
        assert_eq!(deps.len(), 3);
    }

    #[test]
    fn test_blast_radius() {
        let mut graph = SchemaGraph::new();
        
        // System -> Component -> Resource
        graph.add_node("systems/Gate.system.json");
        graph.add_node("components/Store.component.json");
        graph.add_node("resources/db.resource.json");
        
        graph.add_typed_edge("systems/Gate.system.json", "components/Store.component.json", EdgeKind::Requires);
        graph.add_typed_edge("components/Store.component.json", "resources/db.resource.json", EdgeKind::ConnectsTo);
        
        // If db fails, what's affected? (reverse traversal)
        let affected = graph.blast_radius("resources/db.resource.json", &[]);
        assert!(affected.contains("resources/db.resource.json"));
        assert!(affected.contains("components/Store.component.json"));
        assert!(affected.contains("systems/Gate.system.json"));
        assert_eq!(affected.len(), 3);
    }

    #[test]
    fn test_circular_refs() {
        let mut graph = SchemaGraph::new();
        
        // A -> B -> A (circular)
        graph.add_node("A");
        graph.add_node("B");
        graph.add_edge("A", "B");
        graph.add_edge("B", "A");
        
        // Should not infinite loop!
        let deps = graph.transitive_deps(&["A"]);
        assert!(deps.contains("A"));
        assert!(deps.contains("B"));
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_normalize_ref() {
        assert_eq!(normalize_ref("entities/Moment.schema.json", "#/definitions/Foo"), "");
        assert_eq!(
            normalize_ref("entities/Moment.schema.json", "../primitives/Timestamp.schema.json"),
            "primitives/Timestamp.schema.json"
        );
        assert_eq!(
            normalize_ref("entities/Moment.schema.json", "primitives/UUID.schema.json"),
            "primitives/UUID.schema.json"
        );
    }

    #[test]
    fn test_extract_refs() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "id": { "$ref": "primitives/UUID.schema.json" },
                "created_at": { "$ref": "primitives/Timestamp.schema.json" }
            }
        }"#;
        
        let refs = extract_refs(schema);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"primitives/UUID.schema.json".to_string()));
        assert!(refs.contains(&"primitives/Timestamp.schema.json".to_string()));
    }

    #[test]
    fn test_extract_typed_refs() {
        let schema = serde_json::json!({
            "x-familiar-kind": "system",
            "x-familiar-service": { "$ref": "nodes/daemon.node.json" },
            "x-familiar-reads": [
                { "$ref": "entities/Moment.schema.json" }
            ],
            "x-familiar-depends": [
                { "$ref": "components/Store.component.json" }
            ]
        });
        
        let refs = extract_typed_refs(&schema);
        assert_eq!(refs.len(), 3);
        
        assert!(refs.contains(&("nodes/daemon.node.json".to_string(), EdgeKind::RunsOn)));
        assert!(refs.contains(&("entities/Moment.schema.json".to_string(), EdgeKind::Reads)));
        assert!(refs.contains(&("components/Store.component.json".to_string(), EdgeKind::Requires)));
    }

    #[test]
    fn test_edges_of_kind() {
        let mut graph = SchemaGraph::new();
        
        graph.add_node("A");
        graph.add_node("B");
        graph.add_node("C");
        
        graph.add_typed_edge("A", "B", EdgeKind::Reads);
        graph.add_typed_edge("A", "C", EdgeKind::Writes);
        graph.add_typed_edge("B", "C", EdgeKind::TypeRef);
        
        let read_edges = graph.edges_of_kind(&[EdgeKind::Reads]);
        assert_eq!(read_edges.len(), 1);
        
        let data_edges = graph.edges_of_kind(&[EdgeKind::Reads, EdgeKind::Writes]);
        assert_eq!(data_edges.len(), 2);
    }
}

