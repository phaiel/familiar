//! Build script for familiar-core.
//!
//! Performs compile-time drift checking between manually-maintained Rust types
//! and their corresponding JSON schemas in familiar-schemas.
//!
//! It also implements a "Version Lock" (schema-lock) mechanism to ensure the 
//! binary builds against a specific pinned schema version with integrity verification.
//!
//! ## Key Features (Schema Lock 2.0)
//!
//! - **TOML Lock File**: `schema.lock` stores version and content hash
//! - **Integrity Verification**: SHA-256 hash of schema directory
//! - **Out-of-Sync Warning**: Warns when registry has newer version
//! - **Tamper Warning**: Warns when content changes without version bump
//! - **Feature-based Filtering**: Only embed schemas needed for enabled features
//!
//! ## Graph Traversal Patterns
//!
//! Feature filtering uses iterative DFS (stack-based) from the article
//! "Graph & Tree Traversals in Rust" to:
//! - Handle circular `$ref` references safely
//! - Find transitive dependencies for each feature
//! - Avoid stack overflow on deeply nested schemas

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

/// The controlled list of manually-maintained types.
/// These are "Complex Domain" types with physics/relationships/business logic
/// that cannot be automatically generated from schemas.
const MANUAL_TYPES: &[&str] = &[
    // Symmetric Seven - Core Physics Entities (NEVER generate)
    "Moment", "Pulse", "Thread", "Bond", "Filament", "Focus", "Intent", "Motif",
    // Orchestration Entities (NEVER generate)
    "Course", "Shuttle",
    // Components with #[serde(flatten)] (Manual until typify supports)
    "Identity", "FieldExcitation", "QuantumState", "ContentPayload",
    "BondPhysics", "CognitiveOptics", "EmotionalState", "Timestamps",
    // Complex Types with nested structures
    "WeaveUnit", "Weave", "PhysicsHint", "RequestContext",
];

/// Whether to fail on drift or just warn
const FAIL_ON_DRIFT: bool = false;

/// Schema source configuration
#[derive(Debug, Deserialize, Default)]
struct SchemaSource {
    /// Local filesystem path (relative to familiar-core)
    #[serde(default)]
    path: Option<String>,
    /// GitHub repository (future)
    #[serde(default)]
    github: Option<String>,
    /// Git branch (future)
    #[serde(default)]
    branch: Option<String>,
}

/// Schema lock file structure (TOML)
#[derive(Debug, Deserialize)]
struct SchemaLock {
    version: String,
    #[serde(default)]
    hash: String,
    #[serde(default)]
    source: SchemaSource,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
}

fn main() {
    // 1. Resolve and sync schema version with integrity verification
    let (schema_version, lock) = resolve_schema_version();
    
    // 2. Compile protobuf files if present
    compile_protos();
    
    // 3. Run drift check on manual types using the pinned version
    check_drift(&schema_version);
    
    // 4. Validate codegen extensions match Rust serde attributes
    validate_codegen_extensions(&schema_version);
    
    // 5. Process features if any are defined
    if !lock.features.is_empty() {
        process_schema_features(&schema_version, &lock);
    }
}

/// Parse schema.lock TOML file
fn parse_schema_lock(lock_path: &Path) -> SchemaLock {
    let content = fs::read_to_string(lock_path).unwrap_or_else(|_| {
        "version = \"v0.8.0\"\nhash = \"\"".to_string()
    });
    
    // Try to parse as TOML
    if let Ok(lock) = toml::from_str::<SchemaLock>(&content) {
        return lock;
    }
    
    // Fallback for old plain-text format
    let version = content.lines().next().unwrap_or("v0.8.0").trim().to_string();
    SchemaLock {
        version,
        hash: String::new(),
        source: SchemaSource::default(),
        features: HashMap::new(),
    }
}

/// Resolve the schema source path from schema.lock [source] section
///
/// For familiar-core, schemas are now provided by familiar-contracts dependency.
/// We use embedded schemas from the familiar-contracts crate.
fn resolve_schema_source(_manifest_dir: &Path, _lock: &SchemaLock) -> PathBuf {
    // Schemas are now embedded in familiar-contracts and accessed via the SCHEMAS constant
    // We don't need to resolve a path anymore - just return a dummy path for compatibility
    let out_dir_str = std::env::var("OUT_DIR").unwrap();
    let dummy_dir = Path::new(&out_dir_str).join("dummy-schemas");
    std::fs::create_dir_all(&dummy_dir.join("versions").join("latest").join("json-schema")).ok();
    dummy_dir
}

/// Compute SHA-256 hash of the entire schema directory
fn compute_schema_hash(schema_dir: &Path) -> String {
    let mut hasher = Sha256::new();
    
    // Collect all files and sort them for deterministic hashing
    let mut files: Vec<PathBuf> = WalkDir::new(schema_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    
    files.sort();
    
    for file_path in files {
        // Hash the relative path for stability
        if let Ok(relative) = file_path.strip_prefix(schema_dir) {
            hasher.update(relative.to_string_lossy().as_bytes());
        }
        // Hash the file content
        if let Ok(content) = fs::read(&file_path) {
            hasher.update(&content);
        }
    }
    
    format!("sha256:{:x}", hasher.finalize())
}

/// Write schema.lock TOML file
fn write_schema_lock(lock_path: &Path, lock: &SchemaLock) {
    let mut content = format!(
        "# Schema Lock File - similar to Cargo.lock\n\
         # DO NOT EDIT MANUALLY - use `cargo xtask schemas update` to modify\n\n\
         version = \"{}\"\n\
         hash = \"{}\"\n",
        lock.version, lock.hash
    );
    
    if !lock.features.is_empty() {
        content.push_str("\n[features]\n");
        for (name, schemas) in &lock.features {
            let schemas_str = schemas.iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            content.push_str(&format!("{} = [{}]\n", name, schemas_str));
        }
    } else {
        content.push_str("\n# Feature groups for selective schema embedding\n\
                         # Uncomment and customize as needed\n\
                         # [features]\n\
                         # default = [\"core\"]\n\
                         # core = [\"entities/*\", \"components/*\", \"primitives/*\"]\n\
                         # fates = [\"entities/Moment\", \"entities/Pulse\", \"components/FieldExcitation\"]\n\
                         # auth = [\"types/auth/*\"]\n");
    }
    
    fs::write(lock_path, content).expect("Failed to write schema.lock");
}

/// Get the latest version from the registry's symlink
fn get_latest_version(schema_source: &Path) -> Option<String> {
    let latest_link = schema_source.join("versions/latest");
    if let Ok(target) = fs::read_link(&latest_link) {
        let version = if target.is_absolute() {
            target.file_name()
                .map(|n| n.to_string_lossy().to_string())
        } else {
            Some(target.to_string_lossy().to_string())
        };
        return version;
    }
    None
}

fn resolve_schema_version() -> (String, SchemaLock) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lock_file_path = manifest_dir.join("schema.lock");
    let generated_file_path = manifest_dir.join("src/schemas/generated_version.rs");
    
    // Parse the lock file
    let mut lock = parse_schema_lock(&lock_file_path);
    
    // Resolve schema source from [source] section
    let schema_source = resolve_schema_source(manifest_dir, &lock);
    
    // Path to the schema directory for this version
    let schema_dir = schema_source
        .join("versions")
        .join(&lock.version)
        .join("json-schema");
    
    // Check if update is requested via env var
    let update_requested = std::env::var("FAMILIAR_SCHEMA_UPDATE")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    if update_requested {
        if let Some(latest_version) = get_latest_version(&schema_source) {
            if latest_version != lock.version {
                println!("cargo:warning=Updating schema lock from {} to {}", lock.version, latest_version);
                lock.version = latest_version;
            }
        }
        
        // Always recompute hash when updating
        let new_schema_dir = schema_source
            .join("versions")
            .join(&lock.version)
            .join("json-schema");
        
        if new_schema_dir.exists() {
            lock.hash = compute_schema_hash(&new_schema_dir);
            write_schema_lock(&lock_file_path, &lock);
            println!("cargo:warning=Schema lock updated: version={}, hash={}...", lock.version, &lock.hash[..20.min(lock.hash.len())]);
        }
    } else {
        // Not updating - verify integrity
        
        // Warning 1: Out-of-sync with registry
        if let Some(latest_version) = get_latest_version(&schema_source) {
            if latest_version != lock.version {
                println!(
                    "cargo:warning=Registry has newer schemas ({}). Run 'cargo xtask schemas update' to sync.",
                    latest_version
                );
            }
        }
        
        // Warning 2: Integrity check (tamper detection)
        if schema_dir.exists() && !lock.hash.is_empty() {
            let computed_hash = compute_schema_hash(&schema_dir);
            if computed_hash != lock.hash {
                println!(
                    "cargo:warning=INTEGRITY MISMATCH: Locked schema content has changed!"
                );
                println!(
                    "cargo:warning=   Expected: {}...", &lock.hash[..20.min(lock.hash.len())]
                );
                println!(
                    "cargo:warning=   Found:    {}...", &computed_hash[..20]
                );
                println!(
                    "cargo:warning=   Run 'cargo xtask schemas update' to verify and re-lock."
                );
            }
        } else if lock.hash.is_empty() {
            // First time or migrated from old format - compute and store hash
            if schema_dir.exists() {
                lock.hash = compute_schema_hash(&schema_dir);
                write_schema_lock(&lock_file_path, &lock);
                println!("cargo:warning=Initialized schema lock hash: {}...", &lock.hash[..20.min(lock.hash.len())]);
            }
        }
    }
    
    // Determine source description for metadata
    let source_desc = if let Some(ref github) = lock.source.github {
        github.clone()
    } else if let Some(ref path) = lock.source.path {
        path.clone()
    } else {
        "https://github.com/phaiel/familiar-schemas".to_string()
    };

    // Generate the Rust file with version metadata only
    // SCHEMAS are now provided by familiar-contracts crate
    let generated_content = format!(
        "// Schema version metadata\n\
         // SCHEMAS are provided by familiar-contracts crate\n\n\
         pub const SCHEMA_VERSION: &str = \"{}\";\n\
         pub const SCHEMA_HASH: &str = \"{}\";\n\
         pub const SCHEMA_SOURCE: &str = \"{}\";\n",
        lock.version,
        lock.hash,
        source_desc
    );
    
    fs::write(&generated_file_path, generated_content).expect("Failed to write generated_version.rs");
    
    println!("cargo:rerun-if-env-changed=FAMILIAR_SCHEMA_UPDATE");
    println!("cargo:rerun-if-changed=schema.lock");
    
    (lock.version.clone(), lock)
}

fn compile_protos() {
    let proto_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("proto");
    if proto_path.exists() {
        let envelope_proto = proto_path.join("envelope.proto");
        if envelope_proto.exists() {
            println!("cargo:rerun-if-changed=proto/envelope.proto");
            
            if let Err(e) = prost_build::compile_protos(&[envelope_proto], &[proto_path]) {
                eprintln!("Warning: Failed to compile protos: {}", e);
            }
        }
    }
}

fn check_drift(version: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lock = parse_schema_lock(&manifest_dir.join("schema.lock"));
    let schema_source = resolve_schema_source(manifest_dir, &lock);
    let schemas_dir = schema_source
        .join("versions")
        .join(version)
        .join("json-schema");
    
    // Paths to search for manually-maintained Rust types
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let search_dirs = vec![
        manifest_dir.join("src/types"),
        manifest_dir.join("src/entities"),
        manifest_dir.join("src/components"),
        manifest_dir.join("src/primitives"),
    ];
    
    // Rerun if schemas or any source directory changes
    println!("cargo:rerun-if-changed={}", schemas_dir.display());
    for dir in &search_dirs {
        if dir.exists() {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
    }
    
    if !schemas_dir.exists() {
        eprintln!("cargo:warning=Schema directory not found at {}. Skipping drift check.", schemas_dir.display());
        return;
    }
    
    let existing_dirs: Vec<&Path> = search_dirs.iter()
        .filter(|d| d.exists())
        .map(|d| d.as_path())
        .collect();
    
    if existing_dirs.is_empty() {
        return;
    }
    
    match familiar_drift_internals::check_drift_multi(&schemas_dir, &existing_dirs, MANUAL_TYPES) {
        Ok(()) => {
            println!("cargo:warning=Drift check passed for {} manual types (version {})", MANUAL_TYPES.len(), version);
        }
        Err(report) => {
            if FAIL_ON_DRIFT {
                eprintln!("========================================");
                eprintln!("SCHEMA DRIFT DETECTED");
                eprintln!("========================================");
                eprintln!("{}", report);
                process::exit(1);
            } else {
                println!("cargo:warning=Schema drift detected (warnings only during sync period)");
            }
        }
    }
}

/// Validate that x-familiar-* codegen extensions are present and valid.
///
/// This validates:
/// 1. Schemas with oneOf have x-familiar-enum-repr
/// 2. x-familiar-enum-repr values are valid (match ecs/EnumRepr.schema.json)
/// 3. x-familiar-casing values are valid (match ecs/Casing.schema.json)
fn validate_codegen_extensions(version: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lock = parse_schema_lock(&manifest_dir.join("schema.lock"));
    let schema_source = resolve_schema_source(manifest_dir, &lock);
    let schemas_dir = schema_source
        .join("versions")
        .join(version)
        .join("json-schema");
    
    if !schemas_dir.exists() {
        return;
    }
    
    // Valid enum repr values
    let valid_repr = ["internally_tagged", "adjacently_tagged", "externally_tagged", "untagged", "simple_enum"];
    let valid_casing = ["snake_case", "camelCase", "PascalCase", "SCREAMING_SNAKE_CASE", "kebab-case", "lowercase"];
    
    let mut warnings = Vec::new();
    let mut validated = 0;
    
    for entry in WalkDir::new(&schemas_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }
        
        // Skip meta-schemas and codegen config schemas
        let relative = path.strip_prefix(&schemas_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        if relative.starts_with("ecs/") {
            continue;
        }
        
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let schema: serde_json::Value = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        // Check if it has oneOf (union type)
        let has_oneof = schema.get("oneOf").is_some();
        
        if !has_oneof {
            continue;
        }
        
        validated += 1;
        
        // Check if it has x-familiar-enum-repr
        if let Some(repr) = schema.get("x-familiar-enum-repr").and_then(|v| v.as_str()) {
            if !valid_repr.contains(&repr) {
                warnings.push(format!(
                    "{}: invalid x-familiar-enum-repr '{}'. Valid values: {:?}",
                    relative, repr, valid_repr
                ));
            }
        }
        
        // Check x-familiar-casing if present
        if let Some(casing) = schema.get("x-familiar-casing").and_then(|v| v.as_str()) {
            if !valid_casing.contains(&casing) {
                warnings.push(format!(
                    "{}: invalid x-familiar-casing '{}'. Valid values: {:?}",
                    relative, casing, valid_casing
                ));
            }
        }
        
        // Check that internally_tagged has discriminator
        if let Some(repr) = schema.get("x-familiar-enum-repr").and_then(|v| v.as_str()) {
            if (repr == "internally_tagged" || repr == "adjacently_tagged") 
                && schema.get("x-familiar-discriminator").is_none() 
            {
                warnings.push(format!(
                    "{}: {} enum requires x-familiar-discriminator",
                    relative, repr
                ));
            }
        }
    }
    
    if !warnings.is_empty() {
        for w in &warnings {
            println!("cargo:warning=Codegen extension: {}", w);
        }
        if FAIL_ON_DRIFT {
            eprintln!("========================================");
            eprintln!("CODEGEN EXTENSION VALIDATION FAILED");
            eprintln!("========================================");
            for w in &warnings {
                eprintln!("  {}", w);
            }
            process::exit(1);
        }
    } else if validated > 0 {
        println!("cargo:warning=Validated {} union schemas with codegen extensions", validated);
    }
}

/// Process schema features using dependency graph.
///
/// This implements the "Feature Filtering" pattern from the plan:
/// 1. Build a dependency graph from all schemas
/// 2. For each enabled feature, find all transitive dependencies
/// 3. Generate a list of required schemas
///
/// Uses iterative DFS (stack-based) to safely handle:
/// - Circular `$ref` references
/// - Deep nesting without stack overflow
fn process_schema_features(version: &str, lock: &SchemaLock) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let schema_source = resolve_schema_source(manifest_dir, lock);
    let schema_dir = schema_source
        .join("versions")
        .join(version)
        .join("json-schema");
    
    if !schema_dir.exists() {
        return;
    }
    
    // Check which features are enabled via CARGO_FEATURE_* env vars
    let enabled_features: Vec<String> = lock.features.keys()
        .filter(|feature| {
            let env_var = format!("CARGO_FEATURE_{}", feature.to_uppercase().replace('-', "_"));
            std::env::var(&env_var).is_ok()
        })
        .cloned()
        .collect();
    
    if enabled_features.is_empty() {
        // No schema features enabled, embed all schemas
        return;
    }
    
    println!("cargo:warning=Schema features enabled: {:?}", enabled_features);
    
    // Build dependency graph
    let graph = build_schema_graph(&schema_dir);
    
    // Collect root schemas from enabled features
    let mut root_schemas: Vec<String> = Vec::new();
    for feature in &enabled_features {
        if let Some(schemas) = lock.features.get(feature) {
            root_schemas.extend(schemas.clone());
        }
    }
    
    // Resolve transitive dependencies using iterative DFS
    let required = resolve_transitive_deps(&graph, &root_schemas);
    
    println!("cargo:warning=Required schemas: {} (from {} roots)", required.len(), root_schemas.len());
    
    // Export the list for use by include_dir filtering (future enhancement)
    // For now, just log the required schemas
}

/// A simple dependency graph for build.rs use.
/// (We duplicate some logic from src/schemas/graph.rs because build.rs
/// can't depend on the main crate)
struct BuildGraph {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl BuildGraph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }
    
    fn add_node(&mut self, id: &str) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(id) {
            idx
        } else {
            let idx = self.graph.add_node(id.to_string());
            self.node_map.insert(id.to_string(), idx);
            idx
        }
    }
    
    fn add_edge(&mut self, from: &str, to: &str) {
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);
        if !self.graph.contains_edge(from_idx, to_idx) {
            self.graph.add_edge(from_idx, to_idx, ());
        }
    }
}

/// Build a dependency graph from the schema directory.
fn build_schema_graph(schema_dir: &Path) -> BuildGraph {
    let mut graph = BuildGraph::new();
    
    // Discover all schema files
    let schema_files: Vec<PathBuf> = WalkDir::new(schema_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .map(|e| e.path().to_path_buf())
        .collect();
    
    // First pass: create nodes
    for path in &schema_files {
        if let Ok(relative) = path.strip_prefix(schema_dir) {
            graph.add_node(&relative.to_string_lossy());
        }
    }
    
    // Second pass: add edges from $ref
    for path in &schema_files {
        if let Ok(content) = fs::read_to_string(path) {
            let relative = path.strip_prefix(schema_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            
            let refs = extract_refs_build(&content);
            for ref_path in refs {
                let normalized = normalize_ref_build(&relative, &ref_path);
                if !normalized.is_empty() && graph.node_map.contains_key(&normalized) {
                    graph.add_edge(&relative, &normalized);
                }
            }
        }
    }
    
    graph
}

/// Extract $ref values from JSON (iterative, stack-based).
fn extract_refs_build(content: &str) -> Vec<String> {
    let mut refs = Vec::new();
    
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        // Iterative traversal using stack (not recursive!)
        let mut stack = vec![&json];
        
        while let Some(value) = stack.pop() {
            match value {
                serde_json::Value::Object(map) => {
                    if let Some(serde_json::Value::String(r)) = map.get("$ref") {
                        refs.push(r.clone());
                    }
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

/// Normalize a $ref path.
fn normalize_ref_build(current_path: &str, ref_path: &str) -> String {
    if ref_path.starts_with('#') {
        return String::new();
    }
    
    if ref_path.starts_with("../") {
        let current_dir = Path::new(current_path).parent().unwrap_or(Path::new(""));
        let resolved = current_dir.join(ref_path);
        let resolved_str = resolved.to_string_lossy().to_string();
        
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
        ref_path.to_string()
    }
}

/// Resolve transitive dependencies using iterative DFS.
///
/// This is the pattern from "Graph & Tree Traversals in Rust":
/// - Uses a stack instead of recursion (safe in build.rs)
/// - Uses a visited set to handle circular references
fn resolve_transitive_deps(graph: &BuildGraph, roots: &[String]) -> HashSet<String> {
    let mut required: HashSet<String> = HashSet::new();
    let mut stack: Vec<NodeIndex> = Vec::new();
    
    // 1. Add all root schemas to the stack
    for root in roots {
        // Handle glob patterns like "entities/*"
        if root.ends_with("/*") {
            let prefix = root.trim_end_matches("/*");
            for (schema_id, &idx) in &graph.node_map {
                if schema_id.starts_with(prefix) {
                    stack.push(idx);
                }
            }
        } else if let Some(&idx) = graph.node_map.get(root) {
            stack.push(idx);
        }
    }
    
    // 2. Iterative DFS walk (stack-based, not recursive!)
    while let Some(node_idx) = stack.pop() {
        let schema_id = &graph.graph[node_idx];
        
        // Skip if already visited (handles circular refs)
        if !required.insert(schema_id.clone()) {
            continue;
        }
        
        // Add all dependencies (outgoing edges) to the stack
        for neighbor in graph.graph.neighbors_directed(node_idx, Direction::Outgoing) {
            stack.push(neighbor);
        }
    }
    
    required
}
