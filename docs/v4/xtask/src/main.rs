//! xtask - Build automation for Familiar
//!
//! Usage:
//!   cargo xtask schemas export         Export schemas to registry (from familiar-schemas source)
//!   cargo xtask schemas validate       Validate schema compatibility
//!   cargo xtask schemas drift          Check for drift between Rust types and stored schemas
//!   cargo xtask schemas sync           Sync generated types to Windmill workspace
//!   cargo xtask schemas update         Update schema.lock to latest version with integrity hash
//!   cargo xtask schemas graph          Generate dependency graph visualization
//!   cargo xtask schemas validate-graph Validate graph connectivity and extensions
//!
//! ## Schema-First Architecture
//!
//! This project uses a schema-first approach:
//! - Schemas are the source of truth (stored in familiar-schemas)
//! - Rust types must match the schemas (validated by drift check)
//! - TypeScript/Pydantic types are generated FROM schemas (not from Rust)
//!
//! ## Standards Used
//!
//! - **AsyncAPI**: Event-driven API specification (replaces custom YAML)
//! - **CloudEvents**: Message envelope format (CNCF standard)
//! - **JSON Schema**: Type definitions (within AsyncAPI)
//!
//! ## CI Integration
//!
//! Add to your CI pipeline:
//! ```yaml
//! - name: Check schema compliance
//!   run: cargo xtask schemas drift --strict
//!
//! ```

mod sea_codegen;

use clap::{Parser, Subcommand};
#[allow(unused_imports)]
use petgraph::dot::{Config as DotConfig, Dot};
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for Familiar")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Schema management commands (schema-first architecture)
    Schemas {
        #[command(subcommand)]
        action: SchemaCommands,
    },
    /// Code generation commands
    Codegen {
        #[command(subcommand)]
        action: CodegenCommands,
    },
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Export schemas to familiar-schemas registry
    Export {
        /// Version to create (e.g., "0.2.0")
        #[arg(short, long)]
        version: String,
        /// Author name
        #[arg(short, long)]
        author: Option<String>,
        /// Release message
        #[arg(short, long)]
        message: Option<String>,
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Dry run - don't actually register
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate schema compatibility
    Validate {
        /// Base version to compare against
        #[arg(short, long)]
        base: Option<String>,
    },
    /// Check for schema drift between current code and stored schemas
    Drift {
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Version to compare against (default: latest)
        #[arg(short = 'v', long)]
        version: Option<String>,
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Fail on any changes (not just breaking)
        #[arg(long)]
        strict: bool,
        /// Verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Update schema.lock to the latest version in the registry
    Update {
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Generate schema dependency graph
    Graph {
        /// Output file path (default: reports/schemas.dot)
        #[arg(short, long, default_value = "reports/schemas.dot")]
        output: PathBuf,
        /// Also render to PNG (requires graphviz `dot` command)
        #[arg(long)]
        render: bool,
        /// Filter to specific schema prefix (e.g., "entities/")
        #[arg(short, long)]
        filter: Option<String>,
        /// Only include schemas with at least one connection
        #[arg(long)]
        connected_only: bool,
        /// Show statistics about most connected schemas
        #[arg(long)]
        stats: bool,
        /// Enable clustering by directory
        #[arg(long)]
        cluster: bool,
        /// Layout direction: TB (top-bottom), LR (left-right), BT, RL
        #[arg(long, default_value = "LR")]
        layout: String,
        /// Output format: svg, png, pdf
        #[arg(long, default_value = "svg")]
        format: String,
        /// Filter by edge types (comma-separated): type_ref, local_ref, extends, variant_of, union_of, item_type, value_type, field_type, runs_on, uses_queue, requires, reads, writes, connects_to, input, output
        #[arg(long)]
        edge_type: Option<String>,
        /// Depth limit for property traversal (0 = unlimited, 1 = direct refs only)
        #[arg(long, default_value = "0")]
        depth: usize,
        /// Include local definitions (#/definitions/X) as separate nodes
        #[arg(long)]
        include_defs: bool,
    },
    /// Validate schema graph connectivity and x-familiar-* extensions
    ValidateGraph {
        /// Version to validate (default: latest locked version)
        #[arg(short, long)]
        version: Option<String>,
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Lint schema facets for red-line violations (CI-enforced)
    LintFacets {
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Fail on warnings too (strict mode)
        #[arg(long)]
        strict: bool,
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Lint a specific schema by ID
        #[arg(long)]
        schema: Option<String>,
    },
}

#[derive(Subcommand)]
enum CodegenCommands {
    /// Generate Rust types from schemas into contracts/
    Generate {
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Direct path to json-schema directory (bypasses registry/lock)
        #[arg(long)]
        schema_dir: Option<PathBuf>,
        /// Output directory (default: familiar-contracts/src)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Verbose output
        #[arg(long)]
        verbose: bool,
        /// Dry run - show what would be generated
        #[arg(long)]
        dry_run: bool,
    },
    /// Check if generated contracts are up-to-date
    Check {
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Contracts directory (default: familiar-contracts/src)
        #[arg(short, long)]
        contracts: Option<PathBuf>,
    },
    /// Generate TypeScript types from schemas
    Typescript {
        /// Output directory
        #[arg(short, long, default_value = "generated/typescript")]
        output: PathBuf,
    },
    /// Generate Python/Pydantic models from schemas
    Python {
        /// Output directory
        #[arg(short, long, default_value = "generated/python")]
        output: PathBuf,
    },
    /// Generate SeaORM entity files from database schemas
    SeaEntities {
        /// Output directory (default: familiar-core/src/entities/db)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Path to schema registry (default: ../familiar-schemas)
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Validate generated code matches existing hand-written entities
        #[arg(long)]
        validate: bool,
        /// Replace existing hand-written entities with generated code
        #[arg(long)]
        migrate: bool,
    },
    /// Generate SeaORM migrations from entity schemas
    SeaMigration {
        /// Output directory
        #[arg(short, long, default_value = "migration/src")]
        output: PathBuf,
        /// Migration name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Generate API handler stubs from request/response schemas
    ApiHandlers {
        /// Output file
        #[arg(short, long, default_value = "src/api/generated_handlers.rs")]
        output: PathBuf,
    },
    /// Generate OpenAPI spec from schemas
    Openapi {
        /// Output file
        #[arg(short, long, default_value = "openapi.yaml")]
        output: PathBuf,
    },
    /// Generate JSON Schema validators as Rust code
    Validators {
        /// Output file
        #[arg(short, long, default_value = "src/validation/generated.rs")]
        output: PathBuf,
    },
}

/// Schema source configuration
#[derive(Debug, Serialize, Deserialize, Default)]
struct SchemaSource {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    github: Option<String>,
}

/// Schema lock file structure (TOML)
#[derive(Debug, Serialize, Deserialize)]
struct SchemaLock {
    version: String,
    #[serde(default)]
    hash: String,
    #[serde(default)]
    source: SchemaSource,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Schemas { action } => match action {
            SchemaCommands::Export { version, author, message, registry, dry_run } => {
                schemas_export(version, author, message, registry, dry_run)
            },
            SchemaCommands::Validate { base } => schemas_validate(base),
            SchemaCommands::Drift { registry, version, format, strict, verbose } => {
                schemas_drift(registry, version, format, strict, verbose)
            },
            SchemaCommands::Update { registry } => schemas_update(registry),
            SchemaCommands::Graph { output, render, filter, connected_only, stats, cluster, layout, format, edge_type, depth, include_defs } => {
                schemas_graph(output, render, filter, connected_only, stats, cluster, layout, format, edge_type, depth, include_defs)
            },
            SchemaCommands::ValidateGraph { version, registry } => {
                schemas_validate_graph(version, registry)
            },
            SchemaCommands::LintFacets { registry, strict, format, schema } => {
                schemas_lint_facets(registry, strict, format, schema)
            },
        },
        Commands::Codegen { action } => match action {
            CodegenCommands::Generate { registry, schema_dir, output, verbose, dry_run } => {
                codegen_generate(registry, schema_dir, output, verbose, dry_run)
            },
            CodegenCommands::Check { registry, contracts } => {
                codegen_check(registry, contracts)
            },
            CodegenCommands::Typescript { output } => {
                codegen_typescript(output)
            },
            CodegenCommands::Python { output } => {
                codegen_python(output)
            },
            CodegenCommands::SeaEntities { output, registry, validate, migrate } => {
                codegen_sea_entities(output, registry, validate, migrate)
            },
            CodegenCommands::SeaMigration { output, name } => {
                codegen_sea_migration(output, name)
            },
            CodegenCommands::ApiHandlers { output } => {
                codegen_api_handlers(output)
            },
            CodegenCommands::Openapi { output } => {
                codegen_openapi(output)
            },
            CodegenCommands::Validators { output } => {
                codegen_validators(output)
            },
        },
    };
    
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Get the workspace root directory
fn workspace_root() -> anyhow::Result<PathBuf> {
    let output = Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()?;
    
    let path = String::from_utf8(output.stdout)?;
    let cargo_toml = PathBuf::from(path.trim());
    Ok(cargo_toml.parent().unwrap().to_path_buf())
}

/// Get the familiar-schemas registry path
fn get_registry_path(registry: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let root = workspace_root()?;

    // If explicit registry path provided, use it
    if let Some(registry) = registry {
        return Ok(registry);
    }

    // Otherwise, use schema lock configuration
    let lock = parse_lock_file(&root)?;
    match lock.source.github {
        Some(github_url) => {
            // For GitHub sources, Cargo puts the dependency in target/
            let target_dir = root.join("target");
            let repo_name = github_url
                .split('/')
                .last()
                .unwrap_or("familiar-schemas")
                .replace(".git", "");

            Ok(target_dir.join(repo_name))
        }
        None => {
            // Fallback to local path for backward compatibility
            root.parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("familiar-schemas"))
                .ok_or_else(|| anyhow::anyhow!("Could not find familiar-schemas"))
        }
    }
}

/// Parse the schema.lock file
fn parse_lock_file(root: &PathBuf) -> anyhow::Result<SchemaLock> {
    let lock_file = root.join("familiar-core/schema.lock");
    let content = fs::read_to_string(&lock_file)?;
    let lock: SchemaLock = toml::from_str(&content)?;
    Ok(lock)
}

/// Compute SHA-256 hash of schema directory
fn compute_schema_hash(schema_dir: &PathBuf) -> String {
    let mut hasher = Sha256::new();
    
    let mut files: Vec<PathBuf> = WalkDir::new(schema_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    
    files.sort();
    
    for file_path in files {
        if let Ok(relative) = file_path.strip_prefix(schema_dir) {
            hasher.update(relative.to_string_lossy().as_bytes());
        }
        if let Ok(content) = fs::read(&file_path) {
            hasher.update(&content);
        }
    }
    
    format!("sha256:{:x}", hasher.finalize())
}

/// Write schema.lock TOML file
fn write_lock_file(root: &PathBuf, lock: &SchemaLock) -> anyhow::Result<()> {
    let lock_file = root.join("familiar-core/schema.lock");
    
    let mut content = format!(
        "# Schema Lock File - similar to Cargo.lock\n\
         # DO NOT EDIT MANUALLY - use `cargo xtask schemas update` to modify\n\n\
         version = \"{}\"\n\
         hash = \"{}\"\n",
        lock.version, lock.hash
    );
    
    // Write source section (required)
    content.push_str("\n[source]\n");
    if let Some(ref path) = lock.source.path {
        content.push_str(&format!("path = \"{}\"\n", path));
    }
    if let Some(ref github) = lock.source.github {
        content.push_str(&format!("github = \"{}\"\n", github));
    }
    
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
    
    fs::write(lock_file, content)?;
    Ok(())
}

/// Export schemas to the registry
fn schemas_export(
    version: String,
    author: Option<String>,
    message: Option<String>,
    registry: Option<PathBuf>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let _root = workspace_root()?;
    let registry_path = get_registry_path(registry)?;
    
    println!("Exporting schemas to registry...\n");
    println!("   Version:  {}", version);
    println!("   Registry: {}", registry_path.display());
    if dry_run {
        println!("   Mode:     DRY RUN");
    }
    println!();
    
    let mut args = vec![
        "run".to_string(),
        "--bin".to_string(),
        "schema-export".to_string(),
        "-p".to_string(),
        "familiar-schemas".to_string(),
        "--".to_string(),
        "--registry".to_string(),
        registry_path.to_string_lossy().to_string(),
        "--version".to_string(),
        version.clone(),
    ];
    
    if let Some(a) = author {
        args.push("--author".to_string());
        args.push(a);
    }
    
    if let Some(m) = message {
        args.push("--message".to_string());
        args.push(m);
    }
    
    if dry_run {
        args.push("--dry-run".to_string());
    }
    
    let status = Command::new("cargo")
        .args(&args)
        .current_dir(&registry_path)
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to export to registry");
    }
    
    if !dry_run {
        println!("\nSchemas exported to registry version {}", version);
    }
    
    Ok(())
}

/// Validate schema compatibility
fn schemas_validate(base: Option<String>) -> anyhow::Result<()> {
    let registry_path = get_registry_path(None)?;
    
    println!("Validating schema compatibility...\n");
    
    let mut args = vec![
        "run".to_string(),
        "--bin".to_string(),
        "schema-validator".to_string(),
        "-p".to_string(),
        "familiar-schemas".to_string(),
        "--".to_string(),
        "breaking".to_string(),
    ];
    
    if let Some(b) = base {
        args.push("--from".to_string());
        args.push(b);
    }
    
    let status = Command::new("cargo")
        .args(&args)
        .current_dir(&registry_path)
        .status()?;
    
    if status.success() {
        println!("No breaking changes detected");
    } else {
        println!("Breaking changes detected - review before releasing");
    }
    
    Ok(())
}

/// Check for schema drift
fn schemas_drift(
    registry: Option<PathBuf>,
    version: Option<String>,
    format: String,
    strict: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    let root = workspace_root()?;
    let registry_path = get_registry_path(registry)?;
    
    println!("Checking for schema drift...\n");
    
    let mut args = vec![
        "run".to_string(),
        "--bin".to_string(),
        "schema-drift".to_string(),
        "-p".to_string(),
        "familiar-schemas".to_string(),
        "--".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--registry".to_string(),
        registry_path.to_string_lossy().to_string(),
        "--format".to_string(),
        format,
    ];
    
    if let Some(v) = version {
        args.push("--version".to_string());
        args.push(v);
    }
    
    if strict {
        args.push("--strict".to_string());
    }
    
    if verbose {
        args.push("--verbose".to_string());
    }
    
    let status = Command::new("cargo")
        .args(&args)
        .current_dir(&registry_path)
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Schema drift detected!");
    }
    
    Ok(())
}

/// Update schema.lock to the latest version with integrity hash
fn schemas_update(registry: Option<PathBuf>) -> anyhow::Result<()> {
    let root = workspace_root()?;
    let registry_path = get_registry_path(registry)?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              SCHEMA LOCK UPDATE                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Registry: {}", registry_path.display());
    
    // Read current lock
    let current_lock = parse_lock_file(&root).unwrap_or(SchemaLock {
        version: "v0.8.0".to_string(),
        hash: String::new(),
        source: SchemaSource {
            path: Some("../../../familiar-schemas".to_string()),
            github: None,
        },
        features: HashMap::new(),
    });
    
    println!("Current version: {}", current_lock.version);
    if !current_lock.hash.is_empty() {
        println!("Current hash: {}...", &current_lock.hash[..20.min(current_lock.hash.len())]);
    }
    
    // Get latest version from symlink
    let latest_link = registry_path.join("versions/latest");
    let latest_version = if let Ok(target) = fs::read_link(&latest_link) {
        if target.is_absolute() {
            target.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| current_lock.version.clone())
        } else {
            target.to_string_lossy().to_string()
        }
    } else {
        current_lock.version.clone()
    };
    
    println!("Latest version: {}", latest_version);
    
    // Compute new hash
    let schema_dir = registry_path.join("versions").join(&latest_version).join("json-schema");
    if !schema_dir.exists() {
        anyhow::bail!("Schema directory not found: {}", schema_dir.display());
    }
    
    let new_hash = compute_schema_hash(&schema_dir);
    println!("New hash: {}...", &new_hash[..20]);
    
    // Create updated lock (preserve source from current lock)
    let new_lock = SchemaLock {
        version: latest_version.clone(),
        hash: new_hash.clone(),
        source: current_lock.source,
        features: current_lock.features,
    };
    
    // Write lock file
    write_lock_file(&root, &new_lock)?;
    
    println!();
    if latest_version != current_lock.version {
        println!("✅ Updated schema lock: {} -> {}", current_lock.version, latest_version);
    } else if new_hash != current_lock.hash {
        println!("✅ Updated schema hash (version unchanged)");
    } else {
        println!("✅ Schema lock is already up to date");
    }
    
    // Trigger a rebuild to regenerate the version file
    println!();
    println!("Rebuilding familiar-core...");
    let status = Command::new("cargo")
        .args(["build", "-p", "familiar-core"])
        .current_dir(&root)
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to rebuild familiar-core");
    }
    
    println!();
    println!("Done!");

    Ok(())
}

/// Edge types for the schema graph (mirrors EdgeKind in familiar-core)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EdgeType {
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
    RunsOn,
    UsesQueue,
    Requires,
    Reads,
    Writes,
    ConnectsTo,
    Input,
    Output,
}

impl EdgeType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Standard references
            "type_ref" | "typeref" | "ref" => Some(Self::TypeRef),
            "local_ref" | "localref" | "local" => Some(Self::LocalRef),
            // Schema composition
            "extends" | "allof" => Some(Self::Extends),
            "variant_of" | "variantof" | "oneof" => Some(Self::VariantOf),
            "union_of" | "unionof" | "anyof" => Some(Self::UnionOf),
            "item_type" | "itemtype" | "items" => Some(Self::ItemType),
            "value_type" | "valuetype" | "map" => Some(Self::ValueType),
            "field_type" | "fieldtype" | "field" | "property" => Some(Self::FieldType),
            // Infrastructure
            "runs_on" | "runson" => Some(Self::RunsOn),
            "uses_queue" | "usesqueue" => Some(Self::UsesQueue),
            "requires" => Some(Self::Requires),
            "reads" => Some(Self::Reads),
            "writes" => Some(Self::Writes),
            "connects_to" | "connectsto" => Some(Self::ConnectsTo),
            "input" => Some(Self::Input),
            "output" => Some(Self::Output),
            _ => None,
        }
    }

    fn color(&self) -> &'static str {
        match self {
            // Standard references - grays
            Self::TypeRef => "#666666",       // Dark gray (cross-file ref)
            Self::LocalRef => "#AAAAAA",      // Light gray (local ref)
            // Schema composition - spectrum
            Self::Extends => "#4CAF50",       // Green (inheritance)
            Self::VariantOf => "#FF9800",     // Orange (oneOf variant)
            Self::UnionOf => "#FFC107",       // Amber (anyOf union)
            Self::ItemType => "#9C27B0",      // Purple (array item)
            Self::ValueType => "#E91E63",     // Pink (map value)
            Self::FieldType => "#9E9E9E",     // Gray (property field)
            // Infrastructure
            Self::RunsOn => "#2196F3",        // Blue (system -> node)
            Self::UsesQueue => "#673AB7",     // Deep Purple (queue)
            Self::Requires => "#FF5722",      // Deep Orange (component dep)
            Self::Reads => "#00BCD4",         // Cyan (read access)
            Self::Writes => "#F44336",        // Red (write access)
            Self::ConnectsTo => "#03A9F4",    // Light Blue (resource)
            Self::Input => "#8BC34A",         // Light Green (input)
            Self::Output => "#FF5722",        // Deep Orange (output)
        }
    }

    fn label(&self) -> &'static str {
        match self {
            // Standard references
            Self::TypeRef => "ref",
            Self::LocalRef => "local",
            // Schema composition
            Self::Extends => "extends",
            Self::VariantOf => "variant",
            Self::UnionOf => "union",
            Self::ItemType => "item",
            Self::ValueType => "value",
            Self::FieldType => "field",
            // Infrastructure
            Self::RunsOn => "runs_on",
            Self::UsesQueue => "uses_queue",
            Self::Requires => "requires",
            Self::Reads => "reads",
            Self::Writes => "writes",
            Self::ConnectsTo => "connects_to",
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

/// Generate schema dependency graph
fn schemas_graph(
    output: PathBuf,
    render: bool,
    filter: Option<String>,
    connected_only: bool,
    stats: bool,
    cluster: bool,
    layout: String,
    format: String,
    edge_type: Option<String>,
    depth: usize,
    include_defs: bool,
) -> anyhow::Result<()> {
    let root = workspace_root()?;
    let registry_path = get_registry_path(None)?;
    
    // Parse edge type filter
    let edge_filter: Option<std::collections::HashSet<EdgeType>> = edge_type.as_ref().map(|et| {
        et.split(',')
            .filter_map(|s| EdgeType::from_str(s.trim()))
            .collect()
    });
    
    // Parse lock file to get version
    let lock = parse_lock_file(&root)?;
    let schema_dir = registry_path.join("versions").join(&lock.version).join("json-schema");
    
    if !schema_dir.exists() {
        anyhow::bail!("Schema directory not found: {}", schema_dir.display());
    }
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              SCHEMA DEPENDENCY GRAPH                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Schema version: {}", lock.version);
    println!("Source: {}", schema_dir.display());
    println!("Output: {}", output.display());
    if let Some(ref f) = filter {
        println!("Filter: {}", f);
    }
    if let Some(ref ef) = edge_filter {
        println!("Edge types: {:?}", ef.iter().map(|e| e.label()).collect::<Vec<_>>());
    }
    if connected_only {
        println!("Mode: Connected schemas only");
    }
    if cluster {
        println!("Clustering: Enabled (by directory)");
    }
    if depth > 0 {
        println!("Depth: {} levels", depth);
    }
    if include_defs {
        println!("Include local definitions: Yes");
    }
    println!("Layout: {}", layout);
    println!();
    
    // Collect all schema files
    let mut schema_files: Vec<PathBuf> = WalkDir::new(&schema_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .map(|e| e.path().to_path_buf())
        .collect();
    
    schema_files.sort();
    
    println!("Found {} schema files", schema_files.len());
    
    // Build data structures for the graph
    // Map: relative_path -> (display_name, directory)
    let mut schema_info: HashMap<String, (String, String)> = HashMap::new();
    // Map: relative_path -> [(dependency, edge_type)]
    let mut schema_deps: HashMap<String, Vec<(String, EdgeType)>> = HashMap::new();
    
    // First pass: collect all schema info
    for path in &schema_files {
        let relative = path.strip_prefix(&schema_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().to_string());
        
        // Apply filter if specified
        if let Some(ref f) = filter {
            if !relative.starts_with(f) {
                continue;
            }
        }
        
        let display_name = relative
            .replace(".schema.json", "")
            .replace(".component.json", "")
            .replace(".node.json", "")
            .replace(".system.json", "")
            .replace(".resource.json", "")
            .replace(".queue.json", "")
            .split('/')
            .last()
            .unwrap_or(&relative)
            .to_string();
        
        let directory = relative
            .split('/')
            .next()
            .unwrap_or("root")
            .to_string();
        
        schema_info.insert(relative.clone(), (display_name.clone(), directory.clone()));
        
        // Parse the schema file for typed dependencies
        let content = fs::read_to_string(path)?;
        let mut deps: Vec<(String, EdgeType)> = Vec::new();
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Extract typed refs from x-familiar-* extensions
            let typed_refs = extract_typed_refs_for_graph(&json);
            for (ref_path, edge_type) in typed_refs {
                let normalized = normalize_ref(&relative, &ref_path);
                if !normalized.is_empty() {
                    deps.push((normalized, edge_type));
                }
            }
            
            // Extract ALL type refs (including composition: allOf, oneOf, anyOf, items, properties)
            let composition_refs = extract_all_typed_refs_for_graph(&json, &relative, depth, 0);
            for (target, edge_type) in composition_refs {
                // Normalize external refs
                let normalized = if target.contains('#') {
                    // Local definition reference - keep as-is
                    target
                } else {
                    normalize_ref(&relative, &target)
                };
                
                if !normalized.is_empty() && !deps.iter().any(|(d, _)| d == &normalized) {
                    deps.push((normalized, edge_type));
                }
            }
            
            // Add local definitions as separate nodes (if enabled)
            if include_defs {
                let local_defs = extract_definitions_for_graph(&json);
                for def_name in local_defs {
                    let def_id = format!("{}#{}", relative, def_name);
                    let def_display = format!("{}#{}", display_name, def_name);
                    schema_info.insert(def_id.clone(), (def_display, directory.clone()));
                    
                    // Extract refs from this definition
                    let def_schema = json.get("definitions")
                        .or_else(|| json.get("$defs"))
                        .and_then(|defs| defs.get(&def_name));
                    
                    if let Some(def_schema) = def_schema {
                        let def_refs = extract_all_typed_refs_for_graph(def_schema, &relative, depth, 0);
                        let mut def_deps = Vec::new();
                        for (target, edge_type) in def_refs {
                            let normalized = if target.contains('#') {
                                target
                            } else {
                                normalize_ref(&relative, &target)
                            };
                            if !normalized.is_empty() {
                                def_deps.push((normalized, edge_type));
                            }
                        }
                        schema_deps.insert(def_id, def_deps);
                    }
                }
            }
        }
        
        schema_deps.insert(relative, deps);
    }
    
    // Apply edge type filter to deps
    if let Some(ref ef) = edge_filter {
        for (_, deps) in schema_deps.iter_mut() {
            deps.retain(|(_, edge_type)| ef.contains(edge_type));
        }
    }
    
    // Compute connection info for filtering
    let mut connected_schemas: std::collections::HashSet<String> = std::collections::HashSet::new();
    if connected_only {
        for (schema, deps) in &schema_deps {
            if !deps.is_empty() {
                connected_schemas.insert(schema.clone());
                for (dep, _) in deps {
                    connected_schemas.insert(dep.clone());
                }
            }
        }
        println!("Connected schemas: {} (filtered from {})", connected_schemas.len(), schema_info.len());
    }
    
    // Build the petgraph for statistics
    let mut graph: DiGraph<String, ()> = DiGraph::new();
    let mut node_indices: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
    
    // Add nodes (respecting connected_only filter)
    for (relative, (display_name, _)) in &schema_info {
        if connected_only && !connected_schemas.contains(relative) {
            continue;
        }
        let idx = graph.add_node(display_name.clone());
        node_indices.insert(relative.clone(), idx);
    }
    
    // Add edges
    for (relative, deps) in &schema_deps {
        let Some(&from_idx) = node_indices.get(relative) else {
            continue;
        };
        for (dep, _) in deps {
            if let Some(&to_idx) = node_indices.get(dep) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }
    }
    
    println!("Graph: {} nodes, {} edges", graph.node_count(), graph.edge_count());
    
    // Count edges by type
    let mut edge_type_counts: HashMap<EdgeType, usize> = HashMap::new();
    for (_, deps) in &schema_deps {
        for (_, edge_type) in deps {
            *edge_type_counts.entry(*edge_type).or_insert(0) += 1;
        }
    }
    if !edge_type_counts.is_empty() {
        println!();
        println!("Edge types:");
        let mut sorted_counts: Vec<_> = edge_type_counts.iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(a.1));
        for (et, count) in sorted_counts {
            if *count > 0 {
                println!("   {:12} {} edges", format!("{:?}:", et), count);
            }
        }
    }
    
    // Generate custom DOT output with styling (now with typed edges)
    let dot_output = generate_styled_dot_typed(
        &schema_info,
        &schema_deps,
        &connected_schemas,
        connected_only,
        cluster,
        &layout,
    );
    
    fs::write(&output, &dot_output)?;
    println!("✅ Wrote {}", output.display());
    
    // Optionally render to image
    if render {
        let ext = match format.as_str() {
            "png" => "png",
            "pdf" => "pdf",
            _ => "svg",
        };
        let img_path = output.with_extension(ext);
        println!("Rendering to {}...", img_path.display());
        
        let format_arg = format!("-T{}", ext);
        let status = Command::new("dot")
            .args([&format_arg, "-o"])
            .arg(&img_path)
            .arg(&output)
            .status();
        
        match status {
            Ok(s) if s.success() => {
                println!("✅ Rendered {}", img_path.display());
            }
            Ok(_) => {
                println!("⚠ graphviz `dot` command failed");
            }
            Err(_) => {
                println!("⚠ graphviz not installed. Install with:");
                println!("   brew install graphviz  # macOS");
                println!("   apt install graphviz   # Ubuntu");
            }
        }
    }
    
    // Print statistics
    if stats || true { // Always show basic stats
        print_graph_statistics(&graph, &node_indices, &schema_info, stats);
    }
    
    Ok(())
}

/// Print detailed graph statistics
fn print_graph_statistics(
    graph: &DiGraph<String, ()>,
    _node_indices: &HashMap<String, petgraph::graph::NodeIndex>,
    schema_info: &HashMap<String, (String, String)>,
    verbose: bool,
) {
    println!();
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                        STATISTICS                             │");
    println!("└────────────────────────────────────────────────────────────────┘");
    
    // Count schemas by directory
    let mut by_dir: HashMap<&str, usize> = HashMap::new();
    for (_, (_, dir)) in schema_info {
        *by_dir.entry(dir.as_str()).or_default() += 1;
    }
    
    println!("\n📁 Schemas by Directory:");
    let mut sorted_dirs: Vec<_> = by_dir.iter().collect();
    sorted_dirs.sort_by(|a, b| b.1.cmp(a.1));
    for (dir, count) in sorted_dirs {
        println!("   {:15} {:>4} schemas", dir, count);
    }
    
    // Find schemas with most dependencies (out-degree)
    let mut by_deps: Vec<_> = graph.node_indices()
        .map(|idx| {
            let name = graph[idx].clone();
            let deps = graph.neighbors_directed(idx, petgraph::Direction::Outgoing).count();
            (name, deps)
        })
        .filter(|(_, c)| *c > 0)
        .collect();
    by_deps.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n📤 Top Schemas by Dependencies (outgoing edges):");
    let limit = if verbose { 15 } else { 7 };
    for (name, count) in by_deps.iter().take(limit) {
        println!("   {:30} → {} deps", name, count);
    }
    
    // Find most referenced schemas (in-degree)
    let mut by_refs: Vec<_> = graph.node_indices()
        .map(|idx| {
            let name = graph[idx].clone();
            let refs = graph.neighbors_directed(idx, petgraph::Direction::Incoming).count();
            (name, refs)
        })
        .filter(|(_, c)| *c > 0)
        .collect();
    by_refs.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n📥 Most Referenced Schemas (incoming edges):");
    for (name, count) in by_refs.iter().take(limit) {
        println!("   {:30} ← {} refs", name, count);
    }
    
    // Count isolated vs connected nodes (isolated = no incoming AND no outgoing refs)
    let isolated_nodes: Vec<_> = graph.node_indices()
        .filter(|idx| {
            graph.neighbors_directed(*idx, petgraph::Direction::Outgoing).count() == 0 &&
            graph.neighbors_directed(*idx, petgraph::Direction::Incoming).count() == 0
        })
        .map(|idx| graph[idx].clone())
        .collect();
    
    // Count leaf nodes (referenced by others, but don't reference anything - e.g., primitives)
    let leaf_nodes = graph.node_indices()
        .filter(|idx| {
            graph.neighbors_directed(*idx, petgraph::Direction::Outgoing).count() == 0 &&
            graph.neighbors_directed(*idx, petgraph::Direction::Incoming).count() > 0
        })
        .count();
    
    // Count root nodes (reference others, but not referenced by anything)
    let root_nodes = graph.node_indices()
        .filter(|idx| {
            graph.neighbors_directed(*idx, petgraph::Direction::Outgoing).count() > 0 &&
            graph.neighbors_directed(*idx, petgraph::Direction::Incoming).count() == 0
        })
        .count();
    
    println!("\n📊 Connection Summary:");
    println!("   Total schemas:    {}", graph.node_count());
    println!("   Connected:        {}", graph.node_count() - isolated_nodes.len());
    println!("   Leaf nodes:       {} (referenced by others, no outgoing refs)", leaf_nodes);
    println!("   Root nodes:       {} (reference others, not referenced)", root_nodes);
    println!("   Isolated:         {} (no refs in either direction)", isolated_nodes.len());
    println!("   Total edges:      {}", graph.edge_count());
    
    if !isolated_nodes.is_empty() && verbose {
        println!("\n⚠️  Isolated schemas (not connected to graph):");
        for name in &isolated_nodes {
            println!("   • {}", name);
        }
    }
    
    if verbose {
        println!("\n💡 Tips:");
        println!("   • Use --filter <dir> to focus on a specific directory");
        println!("   • Use --connected-only to hide isolated schemas");
        println!("   • Use --cluster to group by directory");
        println!("   • Leaf nodes are typically primitives - highly reused, no dependencies");
        println!("   • Isolated schemas may need $ref integration or be standalone utilities");
    }
}

/// Normalize a $ref path relative to the current schema
fn normalize_ref(current_path: &str, ref_path: &str) -> String {
    // Handle different $ref formats:
    // - "RawSegment.schema.json" (same directory)
    // - "primitives/Timestamp.schema.json" (relative)
    // - "../primitives/Timestamp.schema.json" (parent relative)
    // - "#/definitions/Foo" (local - skip)
    
    if ref_path.starts_with('#') {
        // Local reference, skip
        return String::new();
    }
    
    let current_dir = std::path::Path::new(current_path).parent().unwrap_or(std::path::Path::new(""));
    
    if ref_path.starts_with("../") || !ref_path.contains('/') {
        // Parent-relative path OR same-directory (no slash)
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
        // Already contains a directory path
        ref_path.to_string()
    }
}

/// Extract typed references from x-familiar-* extensions
fn extract_typed_refs_for_graph(json: &serde_json::Value) -> Vec<(String, EdgeType)> {
    let mut refs: Vec<(String, EdgeType)> = Vec::new();
    
    if let serde_json::Value::Object(map) = json {
        // x-familiar-service: { "$ref": "..." } -> RunsOn
        if let Some(service) = map.get("x-familiar-service") {
            if let Some(ref_val) = get_ref_from_value(service) {
                refs.push((ref_val, EdgeType::RunsOn));
            }
        }
        
        // x-familiar-depends: [{ "$ref": "..." }, ...] -> Requires
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-depends") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::Requires));
                }
            }
        }
        
        // x-familiar-resources: [{ "$ref": "..." }, ...] -> ConnectsTo
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-resources") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::ConnectsTo));
                }
            }
        }
        
        // x-familiar-reads: [{ "$ref": "..." }, ...] -> Reads
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-reads") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::Reads));
                }
            }
        }
        
        // x-familiar-writes: [{ "$ref": "..." }, ...] -> Writes
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-writes") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::Writes));
                }
            }
        }
        
        // x-familiar-input: { "$ref": "..." } -> Input
        if let Some(input) = map.get("x-familiar-input") {
            if let Some(ref_val) = get_ref_from_value(input) {
                refs.push((ref_val, EdgeType::Input));
            }
        }
        
        // x-familiar-output: { "$ref": "..." } -> Output
        if let Some(output) = map.get("x-familiar-output") {
            if let Some(ref_val) = get_ref_from_value(output) {
                refs.push((ref_val, EdgeType::Output));
            }
        }
        
        // x-familiar-systems: [{ "$ref": "..." }, ...] -> TypeRef (for nodes)
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-systems") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::TypeRef));
                }
            }
        }
        
        // x-familiar-components: [{ "$ref": "..." }, ...] -> Requires (for nodes)
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-components") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::Requires));
                }
            }
        }
        
        // x-familiar-consumers: [{ "$ref": "..." }, ...] -> UsesQueue
        if let Some(serde_json::Value::Array(arr)) = map.get("x-familiar-consumers") {
            for item in arr {
                if let Some(ref_val) = get_ref_from_value(item) {
                    refs.push((ref_val, EdgeType::UsesQueue));
                }
            }
        }
        
        // x-familiar-config: { "$ref": "..." } -> TypeRef
        if let Some(config) = map.get("x-familiar-config") {
            if let Some(ref_val) = get_ref_from_value(config) {
                refs.push((ref_val, EdgeType::TypeRef));
            }
        }
    }
    
    refs
}

/// Extract ALL type references from a JSON schema, including composition constructs.
/// 
/// This extracts typed edges for:
/// - `$ref` (both local #/definitions/X and external file refs)
/// - `allOf` (inheritance/composition)
/// - `oneOf` (discriminated unions)
/// - `anyOf` (union types)
/// - `items` (array element types)
/// - `additionalProperties` (map value types)
/// - `properties` (object field types)
///
/// The `base_path` is used to create proper local definition IDs.
/// The `depth` parameter controls how deep into properties to traverse (0 = unlimited).
fn extract_all_typed_refs_for_graph(
    schema: &serde_json::Value, 
    base_path: &str,
    depth: usize,
    current_depth: usize,
) -> Vec<(String, EdgeType)> {
    let mut refs = Vec::new();
    
    // Check depth limit for property traversal
    let at_depth_limit = depth > 0 && current_depth >= depth;
    
    if let serde_json::Value::Object(map) = schema {
        // 1. $ref (local or external)
        if let Some(serde_json::Value::String(ref_val)) = map.get("$ref") {
            if ref_val.starts_with("#/definitions/") || ref_val.starts_with("#/$defs/") {
                // Local: #/definitions/Foo -> "file#Foo"
                let def_name = ref_val.split('/').last().unwrap_or("");
                refs.push((format!("{}#{}", base_path, def_name), EdgeType::LocalRef));
            } else if !ref_val.starts_with('#') {
                // External file reference - return raw path (will be normalized later)
                refs.push((ref_val.clone(), EdgeType::TypeRef));
            }
            // Skip other keys in a $ref object (JSON Schema says $ref should be alone)
            return refs;
        }
        
        // 2. allOf -> Extends edges
        if let Some(serde_json::Value::Array(all_of)) = map.get("allOf") {
            for item in all_of {
                let item_refs = extract_all_typed_refs_for_graph(item, base_path, depth, current_depth);
                for (target, kind) in item_refs {
                    // Promote TypeRef/LocalRef to Extends for allOf items
                    let edge_kind = if kind == EdgeType::TypeRef || kind == EdgeType::LocalRef {
                        EdgeType::Extends
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
                let item_refs = extract_all_typed_refs_for_graph(item, base_path, depth, current_depth);
                for (target, kind) in item_refs {
                    let edge_kind = if kind == EdgeType::TypeRef || kind == EdgeType::LocalRef {
                        EdgeType::VariantOf
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
                let item_refs = extract_all_typed_refs_for_graph(item, base_path, depth, current_depth);
                for (target, kind) in item_refs {
                    let edge_kind = if kind == EdgeType::TypeRef || kind == EdgeType::LocalRef {
                        EdgeType::UnionOf
                    } else {
                        kind
                    };
                    refs.push((target, edge_kind));
                }
            }
        }
        
        // 5. items -> ItemType edge
        if let Some(items) = map.get("items") {
            let item_refs = extract_all_typed_refs_for_graph(items, base_path, depth, current_depth);
            for (target, kind) in item_refs {
                let edge_kind = if kind == EdgeType::TypeRef || kind == EdgeType::LocalRef {
                    EdgeType::ItemType
                } else {
                    kind
                };
                refs.push((target, edge_kind));
            }
        }
        
        // 6. additionalProperties -> ValueType edge (if it's a schema, not boolean)
        if let Some(add_props) = map.get("additionalProperties") {
            if add_props.is_object() {
                let prop_refs = extract_all_typed_refs_for_graph(add_props, base_path, depth, current_depth);
                for (target, kind) in prop_refs {
                    let edge_kind = if kind == EdgeType::TypeRef || kind == EdgeType::LocalRef {
                        EdgeType::ValueType
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
                    let field_refs = extract_all_typed_refs_for_graph(field_schema, base_path, depth, current_depth + 1);
                    for (target, kind) in field_refs {
                        // Keep existing kind for nested refs, but use FieldType for direct refs
                        let edge_kind = if kind == EdgeType::TypeRef || kind == EdgeType::LocalRef {
                            EdgeType::FieldType
                        } else {
                            kind
                        };
                        refs.push((target, edge_kind));
                    }
                }
            }
        }
        
        // 8. Recurse into other nested schemas (if, then, else, not)
        // But skip x-familiar-* and definitions (handled separately)
        for (key, value) in map {
            if key.starts_with("x-familiar") {
                continue;
            }
            if key == "definitions" || key == "$defs" {
                continue;
            }
            if key == "properties" || key == "items" || key == "allOf" || 
               key == "oneOf" || key == "anyOf" || key == "additionalProperties" {
                continue;
            }
            if value.is_object() && !at_depth_limit {
                refs.extend(extract_all_typed_refs_for_graph(value, base_path, depth, current_depth));
            }
        }
    }
    
    refs
}

/// Extract local definition names from a schema
fn extract_definitions_for_graph(json: &serde_json::Value) -> Vec<String> {
    let mut defs = Vec::new();
    
    if let serde_json::Value::Object(map) = json {
        // Check for "definitions" (JSON Schema draft-04 to draft-07)
        if let Some(serde_json::Value::Object(definitions)) = map.get("definitions") {
            for def_name in definitions.keys() {
                defs.push(def_name.clone());
            }
        }
        
        // Check for "$defs" (JSON Schema draft 2019-09+)
        if let Some(serde_json::Value::Object(definitions)) = map.get("$defs") {
            for def_name in definitions.keys() {
                defs.push(def_name.clone());
            }
        }
    }
    
    defs
}

/// Extract $ref from a JSON value
fn get_ref_from_value(value: &serde_json::Value) -> Option<String> {
    if let serde_json::Value::Object(map) = value {
        if let Some(serde_json::Value::String(ref_val)) = map.get("$ref") {
            return Some(ref_val.clone());
        }
    }
    None
}

/// Generate styled DOT output with typed edges
fn generate_styled_dot_typed(
    schema_info: &HashMap<String, (String, String)>,
    schema_deps: &HashMap<String, Vec<(String, EdgeType)>>,
    connected_schemas: &std::collections::HashSet<String>,
    connected_only: bool,
    cluster: bool,
    layout: &str,
) -> String {
    let mut dot = String::new();
    
    // Header
    dot.push_str("digraph SchemaGraph {\n");
    dot.push_str(&format!("  rankdir={};\n", layout));
    dot.push_str("  bgcolor=\"#1e1e1e\";\n");
    dot.push_str("  node [shape=box, style=\"filled,rounded\", fontname=\"Helvetica\", fontsize=10, fontcolor=\"white\", color=\"#404040\"];\n");
    dot.push_str("  edge [fontname=\"Helvetica\", fontsize=8, fontcolor=\"#808080\"];\n\n");
    
    // Color map for directories
    let dir_colors: HashMap<&str, &str> = [
        ("primitives", "#607D8B"),    // Blue Gray
        ("entities", "#00BCD4"),      // Cyan
        ("entities_api", "#26C6DA"),  // Light Cyan
        ("tools", "#9C27B0"),         // Purple
        ("auth", "#F44336"),          // Red
        ("components", "#FF9800"),    // Orange
        ("config", "#795548"),        // Brown
        ("ui", "#E91E63"),            // Pink
        ("database", "#3F51B5"),      // Indigo
        ("types", "#009688"),         // Teal
        ("conversation", "#8BC34A"),  // Light Green
        ("agentic", "#FF5722"),       // Deep Orange
        ("windmill", "#673AB7"),      // Deep Purple
        ("nodes", "#2196F3"),         // Blue
        ("systems", "#4CAF50"),       // Green
        ("resources", "#FFC107"),     // Amber
        ("queues", "#9E9E9E"),        // Gray
        ("contracts", "#CDDC39"),     // Lime
        ("ecs", "#00ACC1"),           // Cyan 600
        ("meta", "#78909C"),          // Blue Gray 400
        ("tenant", "#AB47BC"),        // Purple 400
        ("api", "#EF5350"),           // Red 400
    ].into_iter().collect();
    
    if cluster {
        // Group by directory
        let mut by_dir: HashMap<&str, Vec<&String>> = HashMap::new();
        for (relative, (_, dir)) in schema_info {
            if connected_only && !connected_schemas.contains(relative) {
                continue;
            }
            by_dir.entry(dir.as_str()).or_default().push(relative);
        }
        
        for (dir, schemas) in by_dir {
            let color = dir_colors.get(dir).unwrap_or(&"#9E9E9E");
            dot.push_str(&format!("  subgraph cluster_{} {{\n", dir.replace('-', "_")));
            dot.push_str(&format!("    label=\"{}\";\n", dir));
            dot.push_str(&format!("    style=filled;\n"));
            dot.push_str(&format!("    color=\"{}22\";\n", color)); // Add transparency
            dot.push_str(&format!("    fontcolor=\"{}\";\n", color));
            
            for relative in schemas {
                if let Some((display_name, _)) = schema_info.get(relative) {
                    let node_id = relative.replace('/', "_").replace('.', "_").replace('-', "_");
                    dot.push_str(&format!("    \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n", 
                        node_id, display_name, color));
                }
            }
            
            dot.push_str("  }\n\n");
        }
    } else {
        // Non-clustered nodes
        for (relative, (display_name, dir)) in schema_info {
            if connected_only && !connected_schemas.contains(relative) {
                continue;
            }
            let color = dir_colors.get(dir.as_str()).unwrap_or(&"#9E9E9E");
            let node_id = relative.replace('/', "_").replace('.', "_").replace('-', "_");
            dot.push_str(&format!("  \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n", 
                node_id, display_name, color));
        }
        dot.push_str("\n");
    }
    
    // Edges with colors based on type
    for (relative, deps) in schema_deps {
        let from_id = relative.replace('/', "_").replace('.', "_").replace('-', "_");
        
        for (dep, edge_type) in deps {
            if connected_only {
                if !connected_schemas.contains(relative) || !connected_schemas.contains(dep) {
                    continue;
                }
            }
            if !schema_info.contains_key(dep) {
                continue;
            }
            
            let to_id = dep.replace('/', "_").replace('.', "_").replace('-', "_");
            let color = edge_type.color();
            let label = edge_type.label();
            
            if label.is_empty() {
                dot.push_str(&format!("  \"{}\" -> \"{}\" [color=\"{}\"];\n", from_id, to_id, color));
            } else {
                dot.push_str(&format!("  \"{}\" -> \"{}\" [color=\"{}\", label=\"{}\"];\n", from_id, to_id, color, label));
            }
        }
    }
    
    // Legend
    dot.push_str("\n  // Legend\n");
    dot.push_str("  subgraph cluster_legend {\n");
    dot.push_str("    label=\"Edge Types\";\n");
    dot.push_str("    style=filled;\n");
    dot.push_str("    color=\"#2d2d2d\";\n");
    dot.push_str("    fontcolor=\"white\";\n");
    dot.push_str("    node [shape=plaintext];\n");
    dot.push_str("    legend [label=<\n");
    dot.push_str("      <table border=\"0\" cellborder=\"0\" cellspacing=\"4\">\n");
    dot.push_str("        <tr><td><font color=\"#666666\">━━━</font></td><td align=\"left\"><font color=\"white\">type_ref</font></td></tr>\n");
    dot.push_str("        <tr><td><font color=\"#2196F3\">━━━</font></td><td align=\"left\"><font color=\"white\">runs_on</font></td></tr>\n");
    dot.push_str("        <tr><td><font color=\"#9C27B0\">━━━</font></td><td align=\"left\"><font color=\"white\">uses_queue</font></td></tr>\n");
    dot.push_str("        <tr><td><font color=\"#FF9800\">━━━</font></td><td align=\"left\"><font color=\"white\">requires</font></td></tr>\n");
    dot.push_str("        <tr><td><font color=\"#4CAF50\">━━━</font></td><td align=\"left\"><font color=\"white\">reads</font></td></tr>\n");
    dot.push_str("        <tr><td><font color=\"#F44336\">━━━</font></td><td align=\"left\"><font color=\"white\">writes</font></td></tr>\n");
    dot.push_str("        <tr><td><font color=\"#00BCD4\">━━━</font></td><td align=\"left\"><font color=\"white\">connects_to</font></td></tr>\n");
    dot.push_str("      </table>\n");
    dot.push_str("    >];\n");
    dot.push_str("  }\n");
    
    dot.push_str("}\n");
    dot
}

/// Validate schema graph connectivity and x-familiar-* extensions
fn schemas_validate_graph(
    version: Option<String>,
    registry: Option<PathBuf>,
) -> anyhow::Result<()> {
    use petgraph::Direction;
    
    let root = workspace_root()?;
    let registry_path = get_registry_path(registry)?;
    
    // Determine version to validate
    let version = match version {
        Some(v) => v,
        None => {
            let lock = parse_lock_file(&root)?;
            lock.version
        }
    };
    
    let schema_dir = registry_path.join("versions").join(&version).join("json-schema");
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              SCHEMA GRAPH VALIDATION                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Version: {}", version);
    println!("Directory: {}", schema_dir.display());
    println!();
    
    if !schema_dir.exists() {
        anyhow::bail!("Schema directory does not exist: {}", schema_dir.display());
    }
    
    // Collect all schemas and their metadata
    let mut total_schemas = 0;
    let mut schemas_with_kind = 0;
    let mut schemas_without_kind = Vec::new();
    let mut broken_refs = Vec::new();
    let mut orphan_schemas = Vec::new();
    
    // Build a simple graph to check connectivity
    let mut graph: DiGraph<String, ()> = DiGraph::new();
    let mut node_map: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
    
    // First pass: discover all schemas
    for entry in WalkDir::new(&schema_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && 
            e.path().extension().map(|ext| ext == "json").unwrap_or(false)
        })
    {
        let path = entry.path();
        let relative = path.strip_prefix(&schema_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().to_string());
        
        let node_idx = graph.add_node(relative.clone());
        node_map.insert(relative, node_idx);
        total_schemas += 1;
    }
    
    // Second pass: validate and build edges
    for entry in WalkDir::new(&schema_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && 
            e.path().extension().map(|ext| ext == "json").unwrap_or(false)
        })
    {
        let path = entry.path();
        let relative = path.strip_prefix(&schema_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.file_name().unwrap().to_string_lossy().to_string());
        
        let content = fs::read_to_string(path)?;
        let schema: serde_json::Value = serde_json::from_str(&content)?;
        
        // Check for x-familiar-kind
        if schema.get("x-familiar-kind").is_some() {
            schemas_with_kind += 1;
        } else if !relative.starts_with("ecs/") || !relative.contains(".meta.") {
            schemas_without_kind.push(relative.clone());
        }
        
        // Extract all $ref values and check they exist
        let refs = extract_refs_from_schema(&schema);
        for ref_path in refs {
            let normalized = normalize_ref(&relative, &ref_path);
            if !normalized.is_empty() {
                // Check if target exists
                if !node_map.contains_key(&normalized) {
                    broken_refs.push((relative.clone(), normalized.clone()));
                } else {
                    // Add edge
                    if let (Some(&from_idx), Some(&to_idx)) = (node_map.get(&relative), node_map.get(&normalized)) {
                        graph.add_edge(from_idx, to_idx, ());
                    }
                }
            }
        }
    }
    
    // Find orphan schemas (no incoming edges and not a root type)
    for (schema_id, &node_idx) in &node_map {
        let incoming_count = graph.neighbors_directed(node_idx, Direction::Incoming).count();
        
        // Check if it's a "root" schema that shouldn't be referenced
        let is_root = schema_id.starts_with("nodes/") || 
                      schema_id.starts_with("systems/") ||
                      schema_id.starts_with("ecs/");
        
        if incoming_count == 0 && !is_root {
            orphan_schemas.push(schema_id.clone());
        }
    }
    
    // Print validation results
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    VALIDATION RESULTS                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    let mut has_errors = false;
    let mut has_warnings = false;
    
    // Check: All schemas have x-familiar-kind
    println!("✓ x-familiar-kind coverage: {}/{} ({:.1}%)", 
        schemas_with_kind, total_schemas, 
        (schemas_with_kind as f64 / total_schemas as f64) * 100.0);
    
    if !schemas_without_kind.is_empty() {
        has_warnings = true;
        println!();
        println!("⚠ Schemas missing x-familiar-kind ({}):", schemas_without_kind.len());
        for schema in schemas_without_kind.iter().take(10) {
            println!("   - {}", schema);
        }
        if schemas_without_kind.len() > 10 {
            println!("   ... and {} more", schemas_without_kind.len() - 10);
        }
    }
    
    // Check: All $ref targets exist
    if broken_refs.is_empty() {
        println!("✓ All $ref targets exist");
    } else {
        has_errors = true;
        println!();
        println!("✗ Broken $ref references ({}):", broken_refs.len());
        for (schema, ref_path) in broken_refs.iter().take(10) {
            println!("   - {} → {}", schema, ref_path);
        }
        if broken_refs.len() > 10 {
            println!("   ... and {} more", broken_refs.len() - 10);
        }
    }
    
    // Check: No orphan schemas
    println!();
    println!("Graph connectivity:");
    println!("   Nodes: {}", graph.node_count());
    println!("   Edges: {}", graph.edge_count());
    
    if orphan_schemas.is_empty() {
        println!("✓ No orphan schemas (all schemas are reachable)");
    } else {
        has_warnings = true;
        println!();
        println!("⚠ Orphan schemas (no incoming references) ({}):", orphan_schemas.len());
        for schema in orphan_schemas.iter().take(15) {
            println!("   - {}", schema);
        }
        if orphan_schemas.len() > 15 {
            println!("   ... and {} more", orphan_schemas.len() - 15);
        }
    }
    
    // Final verdict
    println!();
    if has_errors {
        println!("❌ VALIDATION FAILED");
        println!();
        println!("Fix the errors above before promoting this version.");
        anyhow::bail!("Validation failed with {} errors", broken_refs.len());
    } else if has_warnings {
        println!("⚠ VALIDATION PASSED WITH WARNINGS");
        println!();
        println!("Consider addressing the warnings above for full graph connectivity.");
    } else {
        println!("✅ VALIDATION PASSED");
        println!();
        println!("This version is ready for promotion.");
    }
    
    Ok(())
}

/// Extract all $ref values from a schema (including nested in x-familiar-* extensions)
fn extract_refs_from_schema(value: &serde_json::Value) -> Vec<String> {
    let mut refs = Vec::new();
    extract_refs_recursive(value, &mut refs);
    refs
}

fn extract_refs_recursive(value: &serde_json::Value, refs: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            // Check for $ref
            if let Some(serde_json::Value::String(ref_val)) = map.get("$ref") {
                refs.push(ref_val.clone());
            }
            // Recurse into all values (including x-familiar-* extensions)
            for v in map.values() {
                extract_refs_recursive(v, refs);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_refs_recursive(v, refs);
            }
        }
        _ => {}
    }
}
/// Lint schema facets for red-line violations
///
/// Enforces the doctrine: "Schemas describe data shape + portable intent;
/// everything executable lives elsewhere."
fn schemas_lint_facets(
    registry: Option<PathBuf>,
    strict: bool,
    format: String,
    schema_id: Option<String>,
) -> anyhow::Result<()> {
    use regex::Regex;
    use std::collections::HashSet;
    
    let registry_path = registry.unwrap_or_else(|| PathBuf::from("../familiar-schemas"));
    let schema_dir = registry_path.join("versions/latest/json-schema");
    
    if !schema_dir.exists() {
        anyhow::bail!("Schema directory not found: {}", schema_dir.display());
    }
    
    // Code-like patterns (Rust, TS, Python syntax)
    let code_patterns: Vec<Regex> = vec![
        Regex::new(r"::\w").unwrap(),           // Rust paths
        Regex::new(r"->").unwrap(),             // Rust/TS arrows
        Regex::new(r"<\w+>").unwrap(),          // Generics
        Regex::new(r"^use\s").unwrap(),         // Rust use
        Regex::new(r"^import\s").unwrap(),      // JS/TS import
        Regex::new(r"^from\s+\w+\s+import").unwrap(), // Python import
        Regex::new(r"fn\s+\w+\s*\(").unwrap(),  // Rust fn
        Regex::new(r"def\s+\w+\s*\(").unwrap(), // Python def
        Regex::new(r"function\s+\w+\s*\(").unwrap(), // JS function
        Regex::new(r"impl\s+\w+").unwrap(),     // Rust impl
        Regex::new(r"trait\s+\w+").unwrap(),    // Rust trait
        Regex::new(r"pub\s+(fn|struct|enum|mod)").unwrap(), // Rust pub
        Regex::new(r"#\[derive").unwrap(),      // Rust derive attr
        Regex::new(r"@\w+\(").unwrap(),         // Decorators
    ];
    
    // Orchestration keywords
    let orchestration_keywords: HashSet<&str> = [
        "workflow", "trigger", "publish", "subscribe", "kafka",
        "temporal", "windmill", "service", "endpoint", "call",
        "invoke", "emit", "dispatch", "queue", "topic",
        "saga", "choreography", "orchestration",
    ].into_iter().collect();
    
    // Validation DSL keywords
    let validation_keywords: HashSet<&str> = [
        "validator", "validate", "constraint", "immutable",
        "non_empty", "valid_email", "regex_match", "cross_field",
        "lifecycle", "conditional_required",
    ].into_iter().collect();
    
    // Allowed x-familiar-* extensions
    let allowed_extensions: HashSet<&str> = [
        // Meta / Classification
        "x-familiar-kind", "x-familiar-deprecated", "x-familiar-role",
        "x-familiar-pii", "x-familiar-pii-class", "x-familiar-meta-schema",
        "x-familiar-requires-auth",
        // Wire / Graph / Infrastructure
        "x-familiar-service", "x-familiar-queue", "x-familiar-resources",
        "x-familiar-depends", "x-familiar-input", "x-familiar-output",
        "x-familiar-reads", "x-familiar-writes",
        // ECS / Node Infrastructure
        "x-familiar-system", "x-familiar-systems", "x-familiar-components",
        "x-familiar-concurrency", "x-familiar-memory", "x-familiar-resource-class",
        // Resource Configuration (structural, not behavioral)
        "x-familiar-resource-type", "x-familiar-virtual", "x-familiar-endpoint",
        "x-familiar-persistence", "x-familiar-tables", "x-familiar-connection-pool",
        "x-familiar-config", "x-familiar-default",
        // Queue Configuration
        "x-familiar-queue-type", "x-familiar-consumers", "x-familiar-producers",
        "x-familiar-dlq", "x-familiar-retention", "x-familiar-visibility-timeout",
        // Rate/Timeout Configuration
        "x-familiar-rate-limit", "x-familiar-timeout", "x-familiar-retries",
        "x-familiar-health-check",
        // LLM Configuration
        "x-familiar-models",
        // Codegen - Portable Intent
        "x-familiar-enum-repr", "x-familiar-discriminator", "x-familiar-content",
        "x-familiar-casing", "x-familiar-variants", "x-familiar-flatten",
        "x-familiar-skip-none", "x-familiar-newtype", "x-familiar-field-alias",
        "x-familiar-field-order", "x-familiar-compose",
        // Codegen - Rust-specific (scoped)
        "x-familiar-rust-impl-ids", "x-familiar-rust-derives",
        "x-familiar-rust-derive-add", "x-familiar-rust-derive-exclude",
        "x-familiar-rust-derive-policy", "x-familiar-rust-default",
        "x-familiar-rust-serde", "x-familiar-rust-recursion",
        // Capabilities (high-level, verifiable)
        "x-familiar-hashable", "x-familiar-orderable", "x-familiar-equality",
    ].into_iter().collect();
    
    struct LintError {
        schema_id: String,
        code: &'static str,
        message: String,
        path: String,
    }
    
    struct LintWarning {
        schema_id: String,
        code: &'static str,
        message: String,
        path: String,
    }
    
    let mut errors: Vec<LintError> = Vec::new();
    let mut warnings: Vec<LintWarning> = Vec::new();
    let mut total_schemas = 0;
    
    fn lint_value(
        value: &serde_json::Value,
        path: &str,
        schema_id: &str,
        code_patterns: &[Regex],
        orchestration_keywords: &HashSet<&str>,
        validation_keywords: &HashSet<&str>,
        allowed_extensions: &HashSet<&str>,
        errors: &mut Vec<LintError>,
        warnings: &mut Vec<LintWarning>,
    ) {
        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    
                    // Check x-familiar-* extensions
                    if key.starts_with("x-familiar-") {
                        // Unknown extension
                        if !allowed_extensions.contains(key.as_str()) {
                            errors.push(LintError {
                                schema_id: schema_id.to_string(),
                                code: "UNKNOWN_EXTENSION",
                                message: format!("Unknown x-familiar-* extension: '{}'", key),
                                path: child_path.clone(),
                            });
                        }
                        
                        // Code in facet
                        if let Some(s) = val.as_str() {
                            for pattern in code_patterns {
                                if pattern.is_match(s) {
                                    errors.push(LintError {
                                        schema_id: schema_id.to_string(),
                                        code: "CODE_IN_FACET",
                                        message: format!("Facet contains code-like string: '{}'", s),
                                        path: child_path.clone(),
                                    });
                                    break;
                                }
                            }
                        }
                        
                        // Orchestration keywords (only check values, not extension names)
                        // Skip: 
                        // - x-familiar-kind (pure classification, can legitimately be "queue", "windmill", etc.)
                        // - x-familiar-queue/service (infrastructure routing, can contain queue/service names)
                        if key != "x-familiar-kind" && !key.contains("queue") && !key.contains("service") {
                            if let Some(s) = val.as_str() {
                                let lower = s.to_lowercase();
                                for kw in orchestration_keywords {
                                    if lower.contains(kw) {
                                        errors.push(LintError {
                                            schema_id: schema_id.to_string(),
                                            code: "ORCHESTRATION_IN_FACET",
                                            message: format!("Facet contains orchestration keyword '{}'", kw),
                                            path: child_path.clone(),
                                        });
                                    }
                                }
                            }
                        }
                        
                        // Validation DSL keywords
                        if let Some(s) = val.as_str() {
                            let lower = s.to_lowercase();
                            for kw in validation_keywords {
                                if lower.contains(kw) {
                                    errors.push(LintError {
                                        schema_id: schema_id.to_string(),
                                        code: "VALIDATION_DSL_IN_FACET",
                                        message: format!("Facet contains validation keyword '{}'", kw),
                                        path: child_path.clone(),
                                    });
                                }
                            }
                        }
                        
                        // No-op defaults
                        let is_noop = match key.as_str() {
                            "x-familiar-rust-derive-policy" => val.as_str() == Some("strict"),
                            "x-familiar-rust-default" => val.as_str() == Some("derived"),
                            "x-familiar-flatten" => val.as_bool() == Some(false),
                            "x-familiar-skip-none" => val.as_bool() == Some(false),
                            "x-familiar-newtype" => val.as_bool() == Some(false),
                            "x-familiar-rust-derive-add" => val.as_array().map(|a| a.is_empty()).unwrap_or(false),
                            "x-familiar-rust-derive-exclude" => val.as_array().map(|a| a.is_empty()).unwrap_or(false),
                            _ => false,
                        };
                        if is_noop {
                            warnings.push(LintWarning {
                                schema_id: schema_id.to_string(),
                                code: "NOOP_FACET",
                                message: format!("Facet '{}' has default value - remove to reduce noise", key),
                                path: child_path.clone(),
                            });
                        }
                    }
                    
                    // Recurse
                    lint_value(val, &child_path, schema_id, code_patterns, orchestration_keywords, validation_keywords, allowed_extensions, errors, warnings);
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let child_path = format!("{}[{}]", path, i);
                    lint_value(val, &child_path, schema_id, code_patterns, orchestration_keywords, validation_keywords, allowed_extensions, errors, warnings);
                }
            }
            _ => {}
        }
    }
    
    // Lint schemas
    for entry in WalkDir::new(&schema_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
    {
        let path = entry.path();
        let rel_path = path.strip_prefix(&schema_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        
        // Filter by schema ID if specified
        if let Some(ref filter_id) = schema_id {
            if !rel_path.contains(filter_id) {
                continue;
            }
        }
        
        let Ok(content) = fs::read_to_string(path) else { continue };
        let Ok(schema): Result<serde_json::Value, _> = serde_json::from_str(&content) else { continue };
        
        total_schemas += 1;
        lint_value(&schema, "", &rel_path, &code_patterns, &orchestration_keywords, &validation_keywords, &allowed_extensions, &mut errors, &mut warnings);
    }
    
    // Output results
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "doctrine": "Schemas describe data shape + portable intent; everything executable lives elsewhere.",
            "total_schemas": total_schemas,
            "total_errors": errors.len(),
            "total_warnings": warnings.len(),
            "errors": errors.iter().map(|e| serde_json::json!({
                "schema": e.schema_id,
                "code": e.code,
                "message": e.message,
                "path": e.path
            })).collect::<Vec<_>>(),
            "warnings": warnings.iter().map(|w| serde_json::json!({
                "schema": w.schema_id,
                "code": w.code,
                "message": w.message,
                "path": w.path
            })).collect::<Vec<_>>()
        }))?);
    } else {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔍 Schema Facet Lint");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("📜 Doctrine: \"Schemas describe data shape + portable intent;");
        println!("   everything executable lives elsewhere.\"");
        println!();
        println!("📊 Summary:");
        println!("   Schemas checked: {}", total_schemas);
        println!("   Errors: {}", errors.len());
        println!("   Warnings: {}", warnings.len());
        println!();
        
        if !errors.is_empty() {
            println!("❌ Errors:");
            for e in &errors {
                println!("   [{:?}] {}:{}", e.code, e.schema_id, e.path);
                println!("      {}", e.message);
            }
            println!();
        }
        
        if !warnings.is_empty() && strict {
            println!("⚠️  Warnings (strict mode):");
            for w in &warnings {
                println!("   [{:?}] {}:{}", w.code, w.schema_id, w.path);
                println!("      {}", w.message);
            }
            println!();
        } else if !warnings.is_empty() {
            println!("⚠️  Warnings ({} total, use --strict to fail):", warnings.len());
            for w in warnings.iter().take(5) {
                println!("   [{:?}] {}:{}", w.code, w.schema_id, w.path);
            }
            if warnings.len() > 5 {
                println!("   ... and {} more", warnings.len() - 5);
            }
            println!();
        }
        
        if errors.is_empty() && (!strict || warnings.is_empty()) {
            println!("✅ PASSED - Schema facets are clean");
        } else {
            println!("❌ FAILED - Fix the errors above");
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
    
    // Exit with error if there are errors (or warnings in strict mode)
    if !errors.is_empty() || (strict && !warnings.is_empty()) {
        anyhow::bail!("Lint failed with {} errors and {} warnings", errors.len(), warnings.len());
    }
    
    Ok(())
}

// =============================================================================
// Codegen Commands
// =============================================================================

/// Generate Rust contracts from JSON schemas
fn codegen_generate(
    registry: Option<PathBuf>,
    schema_dir_override: Option<PathBuf>,
    output: Option<PathBuf>,
    verbose: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let root = workspace_root()?;
    
    // If schema_dir is provided directly, use it (bypasses registry/lock)
    let (schema_dir, version_info) = if let Some(dir) = schema_dir_override {
        (dir, "direct".to_string())
    } else {
        let registry_path = get_registry_path(registry)?;
        let lock = parse_lock_file(&root)?;
        let dir = registry_path
            .join("versions")
            .join(&lock.version)
            .join("json-schema");
        (dir, lock.version)
    };
    
    let output_dir = output.unwrap_or_else(|| root.join("familiar-contracts/src"));
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔧 Generate Rust Contracts (Graph-First)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Schema version: {}", version_info);
    println!("   Schema dir:     {}", schema_dir.display());
    println!("   Output dir:     {}", output_dir.display());
    if verbose { println!("   Verbose:        true"); }
    if dry_run { println!("   Dry run:        true"); }
    println!();
    
    if dry_run {
        println!("   [DRY RUN] Would generate contracts");
        return Ok(());
    }
    
    // Use the new graph-first codegen from familiar-schemas
    // Primitives are auto-detected from directory structure (anything in primitives/ directory)
    let result = familiar_schemas::codegen::generate_rust(&schema_dir)
        .map_err(|diags| anyhow::anyhow!("Codegen failed:\n{}", diags))?;
    
    // Write output
    let output_file = output_dir.join("generated.rs");
    std::fs::write(&output_file, &result.code)?;
    
    println!("   📊 Generated {} types", result.type_count);
    
    if !result.diagnostics.is_empty() {
        println!("   ⚠️  {} warnings", result.diagnostics.warning_count());
        if verbose {
            for item in result.diagnostics.warnings() {
                println!("      - {}", item);
            }
        }
    }
    
    println!("   ✅ Generated contracts to {}", output_file.display());
    println!();
    
    Ok(())
}

/// Check if generated contracts are up-to-date
fn codegen_check(
    registry: Option<PathBuf>,
    contracts: Option<PathBuf>,
) -> anyhow::Result<()> {
    use std::collections::HashSet;
    use sha2::{Sha256, Digest};
    
    let root = workspace_root()?;
    let registry_path = get_registry_path(registry)?;
    let lock = parse_lock_file(&root)?;
    
    let schema_dir = registry_path
        .join("versions")
        .join(&lock.version)
        .join("json-schema");
    
    let contracts_path = contracts.unwrap_or_else(|| root.join("familiar-contracts/src/generated.rs"));
    
    // Compute hash of current schemas
    let mut schema_hash = Sha256::new();
    for entry in walkdir::WalkDir::new(&schema_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
    {
        let content = std::fs::read(entry.path())?;
        schema_hash.update(&content);
    }
    let schema_fingerprint = format!("{:x}", schema_hash.finalize());
    
    // Check if generated file exists and contains the same fingerprint
    if !contracts_path.exists() {
        anyhow::bail!("Contracts file not found at {}. Run `cargo xtask codegen generate`.", contracts_path.display());
    }
    
    let contracts_content = std::fs::read_to_string(&contracts_path)?;
    
    // Look for fingerprint comment in generated file
    // Format: // Schema fingerprint: <hash>
    if !contracts_content.contains(&format!("// Schema fingerprint: {}", &schema_fingerprint[..16])) {
        println!("   Schema fingerprint: {}", &schema_fingerprint[..16]);
        anyhow::bail!("Contracts are out of date. Run `cargo xtask codegen generate` to update.");
    }
    
    println!("   ✅ Contracts are up-to-date (fingerprint: {}...)", &schema_fingerprint[..16]);
    Ok(())
}

// ============================================================================
// NEW CODEGEN TARGETS (TODO: Implement)
// ============================================================================

/// Generate TypeScript types from schemas
/// 
/// Uses ts-rs or custom generator to create TypeScript interfaces
/// that match the Rust types exactly.
fn codegen_typescript(output: PathBuf) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📘 Generate TypeScript Types");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Output: {}", output.display());
    println!();
    
    // TODO: Implement TypeScript generation
    // Options:
    // 1. Use ts-rs crate (already in dependencies)
    // 2. Use typify with TypeScript target
    // 3. Custom JSON Schema -> TypeScript generator
    
    println!("⚠️  Not yet implemented");
    println!();
    println!("   Planned approach:");
    println!("   - Read schemas from registry");
    println!("   - Generate TypeScript interfaces");
    println!("   - Generate Zod validators");
    println!("   - Generate fetch client");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Generate Python/Pydantic models from schemas
/// 
/// Creates Pydantic v2 models that match the Rust types.
fn codegen_python(output: PathBuf) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🐍 Generate Python/Pydantic Models");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Output: {}", output.display());
    println!();
    
    // TODO: Implement Pydantic generation
    // Options:
    // 1. datamodel-codegen (pip install)
    // 2. Custom JSON Schema -> Pydantic generator
    
    println!("⚠️  Not yet implemented");
    println!();
    println!("   Planned approach:");
    println!("   - Read schemas from registry");
    println!("   - Generate Pydantic v2 models");
    println!("   - Generate httpx client");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Generate SeaORM entity files from database schemas
///
/// Reads schemas with x-familiar-kind: "database" and generates entity files
/// matching the existing hand-written pattern.
fn codegen_sea_entities(
    output: Option<PathBuf>,
    registry: Option<PathBuf>,
    validate: bool,
    migrate: bool,
) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🗃️  Generate SeaORM Entities");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    // Resolve paths
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    // schema.lock uses path relative to familiar-core, not workspace
    let schema_registry = registry.unwrap_or_else(|| {
        workspace_root.join("familiar-core").join("../../../familiar-schemas")
    });
    
    // Load schema.lock to get version
    let lock_path = workspace_root.join("familiar-core/schema.lock");
    let lock_content = fs::read_to_string(&lock_path)
        .map_err(|e| anyhow::anyhow!("Failed to read schema.lock: {}", e))?;
    let lock: SchemaLock = toml::from_str(&lock_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse schema.lock: {}", e))?;
    
    let schema_dir = schema_registry
        .join("versions")
        .join(&lock.version)
        .join("json-schema");
    
    let output_dir = output.unwrap_or_else(|| {
        workspace_root.join("familiar-core/src/entities/db/generated")
    });
    
    println!("   Schema version: {}", lock.version);
    println!("   Schema dir:     {}", schema_dir.display());
    println!("   Output dir:     {}", output_dir.display());
    if validate {
        println!("   Mode:           Validate (compare with existing)");
    } else if migrate {
        println!("   Mode:           Migrate (replace hand-written)");
    } else {
        println!("   Mode:           Generate");
    }
    println!();
    
    // Run generation
    let result = sea_codegen::generate_all(&schema_dir, &output_dir, validate)?;
    
    println!("   Parsed:         {} schemas", result.parsed);
    println!("   Generated:      {} files", result.generated.len());
    
    if !result.errors.is_empty() {
        println!();
        println!("   Errors:");
        for err in &result.errors {
            println!("     - {}", err);
        }
    }
    
    if validate {
        println!();
        println!("   Running parity validation...");
        
        let handwritten_dir = workspace_root.join("familiar-core/src/entities/db");
        let parity_result = sea_codegen::validate_parity(&output_dir, &handwritten_dir)?;
        
        println!();
        println!("   Parity Results:");
        println!("   ---------------");
        println!("   Matches:     {} entities", parity_result.matches.len());
        println!("   Mismatches:  {} entities", parity_result.mismatches.len());
        println!("   New:         {} entities", parity_result.new_entities.len());
        
        if !parity_result.mismatches.is_empty() {
            println!();
            println!("   Mismatches:");
            for mismatch in &parity_result.mismatches {
                println!("     - {}", mismatch.file);
                for diff in &mismatch.differences {
                    println!("       {}", diff);
                }
            }
        }
        
        if parity_result.mismatches.is_empty() {
            println!();
            println!("   ✓ All generated entities match hand-written code!");
            println!("   Ready to migrate with: cargo xtask codegen sea-entities --migrate");
        }
    }
    
    if migrate {
        println!();
        println!("   Migrating to generated entities...");
        
        let handwritten_dir = workspace_root.join("familiar-core/src/entities/db");
        
        // Run validation first
        let parity_result = sea_codegen::validate_parity(&output_dir, &handwritten_dir)?;
        
        if !parity_result.mismatches.is_empty() {
            println!();
            println!("   ✗ Cannot migrate: {} mismatches found", parity_result.mismatches.len());
            println!("   Run --validate to see details");
            return Err(anyhow::anyhow!("Parity validation failed"));
        }
        
        // Move generated files to replace hand-written
        println!("   Replacing hand-written entities with generated...");
        
        let modules = ["auth", "conversation", "physics"];
        let preserve_files = ["async_task.rs", "course_trace.rs", "optimistic_lock.rs"];
        
        for module in &modules {
            let gen_module = output_dir.join(module);
            let hw_module = handwritten_dir.join(module);
            
            if gen_module.exists() && hw_module.exists() {
                // Remove old files (except preserved)
                for entry in fs::read_dir(&hw_module)? {
                    let entry = entry?;
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    
                    if !preserve_files.contains(&file_name.as_str()) {
                        fs::remove_file(entry.path())?;
                    }
                }
                
                // Copy new files
                for entry in fs::read_dir(&gen_module)? {
                    let entry = entry?;
                    let dest = hw_module.join(entry.file_name());
                    fs::copy(entry.path(), &dest)?;
                }
            }
        }
        
        // Handle task and trace modules (preserve, don't replace)
        let task_dir = handwritten_dir.join("task");
        let trace_dir = handwritten_dir.join("trace");
        
        // Remove generated directory
        fs::remove_dir_all(&output_dir)?;
        
        println!();
        println!("   ✓ Migration complete!");
        println!("   - Replaced: auth/, conversation/, physics/");
        println!("   - Preserved: task/async_task.rs, trace/course_trace.rs, optimistic_lock.rs");
        println!();
        println!("   Next: Update familiar-core/src/entities/db/mod.rs if needed");
    }
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if result.errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Generation completed with {} errors", result.errors.len()))
    }
}

/// Generate SeaORM migrations from entity schemas
/// 
/// Creates migration files for entities with x-familiar-kind: "entity"
fn codegen_sea_migration(output: PathBuf, name: Option<String>) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🗃️  Generate SeaORM Migration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Output: {}", output.display());
    if let Some(ref n) = name {
        println!("   Name:   {}", n);
    }
    println!();
    
    // TODO: Implement SeaORM migration generation
    // - Read entity schemas (x-familiar-kind: "entity")
    // - Map JSON Schema types to SeaORM column types
    // - Generate migration file
    
    println!("⚠️  Not yet implemented");
    println!();
    println!("   Planned approach:");
    println!("   - Find schemas with x-familiar-kind: entity");
    println!("   - Map schema properties to SeaORM columns");
    println!("   - Generate create_table migration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Generate API handler stubs from request/response schemas
/// 
/// Creates axum handler functions for schemas matching API patterns.
fn codegen_api_handlers(output: PathBuf) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🌐 Generate API Handler Stubs");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Output: {}", output.display());
    println!();
    
    // TODO: Implement API handler generation
    // - Find *Input/*Response schema pairs
    // - Generate axum handler with correct types
    // - Include validation middleware
    
    println!("⚠️  Not yet implemented");
    println!();
    println!("   Planned approach:");
    println!("   - Find CreateXInput + XResponse pairs");
    println!("   - Generate async fn create_x(Json<CreateXInput>) -> Result<Json<XResponse>>");
    println!("   - Wire up validation middleware");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Generate OpenAPI specification from schemas
/// 
/// Creates an OpenAPI 3.1 spec that documents the API.
fn codegen_openapi(output: PathBuf) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 Generate OpenAPI Specification");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Output: {}", output.display());
    println!();
    
    // TODO: Implement OpenAPI generation
    // - Use utoipa or custom generator
    // - Include all API schemas
    // - Generate paths from handler patterns
    
    println!("⚠️  Not yet implemented");
    println!();
    println!("   Planned approach:");
    println!("   - Collect API schemas");
    println!("   - Generate OpenAPI 3.1 paths");
    println!("   - Include $ref to schema components");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Generate JSON Schema validators as Rust code
/// 
/// Embeds schema validation as compiled Rust code for runtime validation.
fn codegen_validators(output: PathBuf) -> anyhow::Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Generate Validators");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Output: {}", output.display());
    println!();
    
    // TODO: Implement validator generation
    // - Use jsonschema crate
    // - Embed schema as static JSON
    // - Generate validate_X() functions
    
    println!("⚠️  Not yet implemented");
    println!();
    println!("   Planned approach:");
    println!("   - For each API input schema:");
    println!("   - Generate: pub fn validate_create_x(input: &CreateXInput) -> Result<(), ValidationError>");
    println!("   - Use jsonschema crate for validation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}
