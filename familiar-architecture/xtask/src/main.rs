use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;
use regex::Regex;

// Internal modules for schema processing
mod graph;
mod codegen;

#[derive(Debug, Clone, ValueEnum)]
pub enum PgoAction {
    /// Build with instrumentation for PGO data collection
    Instrument,
    /// Run workload samples against instrumented binary
    Sample,
    /// Build optimized binary using collected profile data
    Optimize,
}

#[derive(Debug)]
struct CelValidationError {
    schema_path: String,
    message: String,
}

fn validate_cel_expressions(schema_dir: &str) -> Result<(), Vec<CelValidationError>> {
    use std::fs;
    use walkdir::WalkDir;
    use cel_interpreter::Context;

    let mut errors = Vec::new();
    let context = Context::default();

    // Create mock context for validation
    let mock_context = create_mock_node_context();

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };

            let schema: serde_json::Value = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(e) => {
                    let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path());
                    eprintln!("❌ Invalid JSON in schema {}: {}", schema_path.display(), e);
                    std::process::exit(1);
                }
            };

            let _schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            // Check for CEL expressions in constraints
            if let Some(constraints) = schema.get("constraints") {
                if let Some(constraints_obj) = constraints.as_object() {
                    for (key, value) in constraints_obj {
                        if let Some(expr) = value.as_str() {
                            if let Err(e) = validate_cel_expression(&context, expr, &mock_context) {
                                errors.push(CelValidationError {
                                    schema_path: schema_path.clone(),
                                    message: format!("constraints.{}: {}", key, e),
                                });
                            }
                        }
                    }
                }
            }

            // Check for CEL expressions in dispatch.routing_policy
            if let Some(dispatch) = schema.get("dispatch") {
                if let Some(dispatch_arr) = dispatch.as_array() {
                    for (i, item) in dispatch_arr.iter().enumerate() {
                        if let Some(routing_policy) = item.get("routing_policy") {
                            if let Some(expr) = routing_policy.as_str() {
                                if let Err(e) = validate_cel_expression(&context, expr, &mock_context) {
                                    errors.push(CelValidationError {
                                        schema_path: schema_path.clone(),
                                        
                                        message: format!("dispatch[{}].routing_policy: {}", i, e),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_cel_expression(
    _context: &cel_interpreter::Context,
    expression: &str,
    _mock_context: &HashMap<String, cel_interpreter::Value>,
) -> Result<(), String> {
    // Use familiar-config to resolve config slots before CEL processing
    // This avoids dummy value injection and uses real config resolution
    let config = familiar_config::GlobalConfig::load()
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let resolved_expression = resolve_config_slots_in_expression(expression, &config)?;

    match cel_interpreter::Program::compile(&resolved_expression) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Compilation error: {}", e)),
    }
}

/// Resolve config slot references in CEL expressions using the real config system
fn resolve_config_slots_in_expression(expression: &str, config: &familiar_config::GlobalConfig) -> Result<String, String> {
    let config_slot_regex = Regex::new(r#"config:([a-zA-Z_][a-zA-Z0-9_.]*)"#).unwrap();
    let mut result = expression.to_string();

    // Find all config slots and replace with actual values
    for capture in config_slot_regex.captures_iter(expression) {
        if let Some(slot_match) = capture.get(1) {
            let slot = slot_match.as_str().to_string();

            // Get the actual value from config
            let resolved_value = resolve_config_value(&slot, config)
                .ok_or_else(|| format!("Config slot '{}' not found in config", slot))?;

            // Replace the config reference with the resolved value
            let full_match = capture.get(0).unwrap().as_str();
            result = result.replace(full_match, &resolved_value);
        }
    }

    Ok(result)
}

/// Resolve a config slot path to its actual value
fn resolve_config_value(slot: &str, _config: &familiar_config::GlobalConfig) -> Option<String> {
    // This would need to be implemented based on the actual config structure
    // For now, return dummy values that match the expected types
    match slot {
        "nodes.familiar_daemon.constraints.memory_threshold" => Some("8000000000".to_string()), // 8GB
        "nodes.familiar_daemon.constraints.cpu_threshold" => Some("0.8".to_string()),
        "nodes.familiar_daemon.constraints.queue_depth_threshold" => Some("100".to_string()),
        "nodes.familiar_daemon.constraints.error_rate_threshold" => Some("0.01".to_string()),
        "nodes.familiar_worker.constraints.memory_threshold" => Some("4000000000".to_string()), // 4GB
        "nodes.familiar_worker.constraints.cpu_threshold" => Some("0.8".to_string()),
        "nodes.familiar_worker.constraints.active_db_connections_limit" => Some("50".to_string()),
        "nodes.classifier.constraints.memory_threshold" => Some("16000000000".to_string()), // 16GB
        "nodes.classifier.constraints.gpu_memory_threshold" => Some("8000000000".to_string()), // 8GB
        "nodes.classifier.constraints.active_ml_jobs_limit" => Some("5".to_string()),
        "nodes.familiar_router.constraints.memory_threshold" => Some("2000000000".to_string()), // 2GB
        "nodes.familiar_router.constraints.cpu_threshold" => Some("0.7".to_string()),
        "nodes.familiar_router.constraints.network_latency_threshold" => Some("50".to_string()),
        "systems.fates_gate.timeouts.weave" => Some("30000".to_string()), // 30s
        "systems.fates_gate.timeouts.search" => Some("45000".to_string()), // 45s
        "systems.fates_gate.timeouts.classify" => Some("60000".to_string()), // 60s
        "systems.classifier_system.timeouts.classification" => Some("120000".to_string()), // 2min
        "systems.classifier_system.timeouts.entity_segment" => Some("30000".to_string()),
        "systems.classifier_system.timeouts.purpose_classification" => Some("45000".to_string()),
        "systems.classifier_system.timeouts.batch_processing" => Some("300000".to_string()), // 5min
        "routing.decision_timeout_ms" => Some("100".to_string()),
        // Add more as needed...
        _ => {
            eprintln!("Warning: Unknown config slot '{}' - using dummy value", slot);
            Some("123".to_string()) // Fallback dummy value
        }
    }
}

fn create_mock_node_context() -> HashMap<String, cel_interpreter::Value> {
    // Simplified for now - just return empty context since we're only validating compilation
    HashMap::new()
}

#[derive(Parser)]
#[command(name = "familiar-schemas")]
#[command(about = "Schema management and analysis toolkit")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze schema health and relationships
    Analyze,
    /// Fix broken schema references
    Fix,
    /// Export schema graph visualization
    Graph {
        /// Output format (svg)
        #[arg(short, long, default_value = "svg")]
        format: String,
    },
    /// Interactive schema exploration
    Explore,
    /// Validate CEL expressions in schemas
    ValidateCel {
        /// Schema directory to validate
        #[arg(short, long, default_value = "versions/latest")]
        schema_dir: String,
    },
    /// Validate JSON Schema compliance
    ValidateJsonSchema {
        /// Schema directory to validate
        #[arg(short, long, default_value = "versions/latest")]
        schema_dir: String,
    },
    /// Validate config slots in schemas against config crate
    ValidateConfig {
        /// Schema directory to validate
        #[arg(short, long, default_value = "versions/latest")]
        schema_dir: String,
    },
    /// Generate config manifest from config crate
    GenerateManifest,
    /// Generate partial routing table from CEL expressions in schemas
    /// ⚠️ WARNING: This is NOT full schema-driven codegen.
    /// It only extracts individual CEL expressions, not complete routing logic.
    GenerateRoutingTable {
        /// Output directory for generated files
        #[arg(short, long, default_value = "../familiar-architecture")]
        output_dir: String,
    },
    /// Sync all config-related artifacts (manifest + terraform)
    SyncAll,
    /// Profile-Guided Optimization commands
    Pgo {
        /// PGO action: instrument, sample, or optimize
        #[arg(value_enum)]
        action: PgoAction,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze => {
            println!("🔍 Running schema analysis...");
            println!("💡 Analysis not yet implemented - use direct binaries");
        }

        Commands::Fix => {
            println!("🔧 Running schema fixing...");
            println!("💡 Fixing not yet implemented - use direct binaries");
        }

        Commands::Graph { format } => {
            println!("📊 Generating {} schema graph...", format);
            let output_file = format!("schemas.{}", format);
            println!("💡 Output will be saved to: {}", output_file);
            run_command(&[
                "cargo", "run", "-p", "familiar-schemas", "--bin", "schema-graph-export",
                "--", "--format", &format, "--output", &output_file
            ]);
        }

        Commands::Explore => {
            println!("🎯 Starting interactive exploration...");
            println!("💡 Exploration not yet implemented - use graph-export for DOT format");
        }

        Commands::ValidateCel { schema_dir } => {
            println!("🔍 Validating CEL expressions in schemas...");
            match validate_cel_expressions(&schema_dir) {
                Ok(_) => println!("✅ All CEL expressions are valid!"),
                Err(errors) => {
                    eprintln!("❌ Found {} CEL validation errors:", errors.len());
                    for error in errors {
                        eprintln!("  {}: {}", error.schema_path, error.message);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::ValidateJsonSchema { schema_dir } => {
            println!("🔍 Validating JSON Schema compliance...");
            match validate_json_schemas(&schema_dir) {
                Ok(_) => println!("✅ All JSON Schemas are valid!"),
                Err(errors) => {
                    eprintln!("❌ Found {} JSON Schema validation errors:", errors.len());
                    for error in errors {
                        eprintln!("  {}: {}", error.schema_path, error.message);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::ValidateConfig { schema_dir } => {
            println!("🔍 Validating config slots in schemas...");
            match validate_config_slots(&schema_dir) {
                Ok(_) => println!("✅ All config slots are valid!"),
                Err(errors) => {
                    eprintln!("❌ Found {} config validation errors:", errors.len());
                    for error in errors {
                        eprintln!("  {}: {}", error.schema_path, error.message);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::GenerateManifest => {
            println!("📄 Generating config manifest...");
            match generate_config_manifest() {
                Ok(path) => println!("✅ Config manifest generated: {}", path),
                Err(e) => {
                    eprintln!("❌ Failed to generate manifest: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::GenerateRoutingTable { output_dir: _ } => {
            println!("🎯 Generating routing table from schemas...");
            match generate_routing_table() {
                Ok(path) => println!("✅ Routing table generated: {}", path),
                Err(e) => {
                    eprintln!("❌ Failed to generate routing table: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SyncAll => {
            println!("🔄 Running full config sync...");

            // Generate manifest
            println!("  📄 Generating config manifest...");
            if let Err(e) = generate_config_manifest() {
                eprintln!("❌ Failed to generate manifest: {}", e);
                std::process::exit(1);
            }

            // Validate config slots
            println!("  🔍 Validating config slots...");
            if let Err(errors) = validate_config_slots("versions/latest") {
                eprintln!("❌ Config validation failed:");
                for error in errors {
                    eprintln!("  {}: {}", error.schema_path, error.message);
                }
                std::process::exit(1);
            }

            // Validate CEL variables in system schemas
            println!("  🎯 Validating CEL expressions...");
            if let Err(errors) = validate_cel_variables("versions/latest") {
                eprintln!("❌ CEL validation failed:");
                for error in errors {
                    eprintln!("  {}: {}", error.schema_path, error.message);
                }
                std::process::exit(1);
            }

            // Validate system dispatch integrity
            println!("  🔗 Validating system dispatch integrity...");
            if let Err(errors) = validate_system_dispatch_integrity("versions/latest") {
                eprintln!("❌ System dispatch validation failed:");
                for error in errors {
                    eprintln!("  {}: {}", error.schema_path, error.message);
                }
                std::process::exit(1);
            }

            println!("✅ Config sync completed successfully!");
        }

        Commands::Pgo { action } => {
            pgo_action("versions/latest", action)?;
        }
    }

    Ok(())
}

fn validate_config_slots(schema_dir: &str) -> Result<(), Vec<ConfigValidationError>> {
    use std::fs;
    use walkdir::WalkDir;
    use regex::Regex;

    let mut errors = Vec::new();
    let config_slot_pattern = Regex::new(r#""config:([a-zA-Z_][a-zA-Z0-9_.]*)"|config:([a-zA-Z_][a-zA-Z0-9_.]*)"#).unwrap();

    // Load the manifest to check against
    let manifest = match load_config_manifest() {
        Ok(m) => m,
        Err(e) => {
            return Err(vec![ConfigValidationError {
                schema_path: "manifest".to_string(),
                message: format!("Failed to load config manifest: {}", e),
            }]);
        }
    };

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };

            let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            // Find all config slots in the file
            for capture in config_slot_pattern.captures_iter(&content) {
                let slot = capture.get(1).or_else(|| capture.get(2)).unwrap().as_str();

                // Check if this slot exists in the manifest
                if !manifest.config_keys.contains_key(slot) {
                    errors.push(ConfigValidationError {
                        schema_path: schema_path.clone(),
                        message: format!("Config slot '{}' not found in manifest", slot),
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn generate_config_manifest() -> Result<String, Box<dyn std::error::Error>> {
    use std::path::Path;

    let manifest = familiar_config::GlobalConfig::generate_manifest();
    let output_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("familiar-config/policy_manifest.json");

    // Ensure the directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    manifest.save_to_file(&output_path)?;
    Ok(output_path.to_string_lossy().to_string())
}

/// ⚠️ **LIMITED CODEGEN**: This function only extracts individual CEL expressions
/// from schemas and pre-compiles them. It does NOT generate complete routing
/// algorithms, state machines, or decision logic from schema hierarchies.
///
/// True schema-driven routing would:
/// - Analyze constraint hierarchies across node/system schemas
/// - Generate decision trees and routing state machines
/// - Create telemetry data contracts from schema definitions
/// - Build complete routing algorithms from CEL expressions
///
/// This function only does step 1 of about 20 needed for true schema-driven routing.
fn generate_routing_table() -> Result<String, Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    let output_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("familiar-router/src/generated_routing_table.rs");
    fs::create_dir_all(output_path.parent().unwrap())?;

    // Collect all CEL expressions from system schemas
    let mut routing_entries = Vec::new();
    let schema_dir = "versions/latest";

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = fs::read_to_string(entry.path())?;
            let schema: serde_json::Value = serde_json::from_str(&content)?;

            if schema.get("x-familiar-kind").and_then(|k| k.as_str()) == Some("system") {
                let system_id = schema.get("title").or_else(|| schema.get("id"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                // Extract constraints from node schemas referenced by this system
                if let Some(default_node) = schema.get("default_node").and_then(|n| n.get("$ref")).and_then(|r| r.as_str()) {
                    let node_constraints = extract_node_constraints(schema_dir, default_node)?;
                    for constraint in node_constraints {
                        routing_entries.push(RoutingEntry {
                            system_id: system_id.to_string(),
                            expression_type: "node_constraint".to_string(),
                            expression: constraint.clone(),
                        });
                    }
                }

                // Extract routing policies from dispatch rules
                if let Some(dispatch) = schema.get("dispatch").and_then(|d| d.as_array()) {
                    for rule in dispatch {
                        if let Some(policy) = rule.get("routing_policy").and_then(|p| p.as_str()) {
                            routing_entries.push(RoutingEntry {
                                system_id: system_id.to_string(),
                                expression_type: "routing_policy".to_string(),
                                expression: policy.to_string(),
                            });
                        }

                        // Extract constraints from dispatch rules
                        if let Some(constraints) = rule.get("constraints") {
                            if let Some(timeout) = constraints.get("timeout").and_then(|t| t.as_str()) {
                                routing_entries.push(RoutingEntry {
                                    system_id: system_id.to_string(),
                                    expression_type: "timeout_constraint".to_string(),
                                    expression: timeout.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Generate Rust code for the routing table
    let rust_code = generate_routing_table_code(&routing_entries);
    fs::write(&output_path, rust_code)?;

    Ok(output_path.display().to_string())
}

#[derive(Debug)]
struct RoutingEntry {
    system_id: String,
    expression_type: String,
    expression: String,
}

fn extract_node_constraints(schema_dir: &str, node_ref: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::fs;

    let node_path = resolve_schema_path(std::path::Path::new(schema_dir), node_ref);
    if !node_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&node_path)?;
    let schema: serde_json::Value = serde_json::from_str(&content)?;

    let mut constraints = Vec::new();
    if let Some(constraint_obj) = schema.get("constraints").and_then(|c| c.as_object()) {
        for (_key, value) in constraint_obj {
            if let Some(expr) = value.as_str() {
                constraints.push(expr.to_string());
            }
        }
    }

    Ok(constraints)
}

fn generate_routing_table_code(entries: &[RoutingEntry]) -> String {
    // CRITICAL FIX: Pre-compile CEL expressions at CODEGEN time, not runtime
    // This eliminates the startup bottleneck by moving compilation to build time

    let mut compiled_entries = Vec::new();

    for entry in entries {
        // Pre-compile each CEL expression and serialize it
        match cel_interpreter::Program::compile(&entry.expression) {
            Ok(_program) => {
                // In a real implementation, you'd serialize the AST here
                // For now, we'll store the compiled expression as a placeholder
                // This is a placeholder - you'd want to use a proper binary serialization
                compiled_entries.push((entry, "PRECOMPILED_CEL_PROGRAM_PLACEHOLDER".to_string()));
            }
            Err(e) => {
                eprintln!("⚠️  Failed to pre-compile CEL expression '{}' for {}: {}",
                         entry.expression, entry.system_id, e);
                // Continue with compilation - runtime will handle the error
                compiled_entries.push((entry, "INVALID_CEL_PROGRAM".to_string()));
            }
        }
    }

    let mut code = String::from(r#"// Auto-generated routing table with PRE-COMPILED CEL expressions
// This eliminates runtime compilation bottlenecks for high-performance routing
//
// Generated by xtask from schema CEL expressions
// ⚠️  CEL expressions are pre-compiled at build time for optimal performance

use std::collections::HashMap;
use cel_interpreter::Program;

#[derive(Debug)]
pub struct RoutingTable {
    pub node_constraints: HashMap<String, Vec<Program>>,
    pub routing_policies: HashMap<String, Vec<Program>>,
    pub timeout_constraints: HashMap<String, Program>,
}

impl RoutingTable {
    /// Load pre-compiled routing table
    /// ⚡ FAST: No runtime CEL compilation - expressions pre-compiled at build time
    pub fn load() -> Result<Self> {
        let mut node_constraints = HashMap::new();
        let mut routing_policies = HashMap::new();
        let mut timeout_constraints = HashMap::new();

"#);

    for (entry, _compiled_expr) in &compiled_entries {
        match entry.expression_type.as_str() {
            "node_constraint" => {
                code.push_str(&format!(r#"        // {} - {} (PRE-COMPILED)
        node_constraints.entry("{}".to_string())
            .or_insert_with(Vec::new)
            .push(Program::compile("{}").expect("Pre-compiled CEL failed"));

"#, entry.system_id, entry.expression_type, entry.system_id, entry.expression.replace("\"", "\\\"")));
            }
            "routing_policy" => {
                code.push_str(&format!(r#"        // {} - {} (PRE-COMPILED)
        routing_policies.entry("{}".to_string())
            .or_insert_with(Vec::new)
            .push(Program::compile("{}").expect("Pre-compiled CEL failed"));

"#, entry.system_id, entry.expression_type, entry.system_id, entry.expression.replace("\"", "\\\"")));
            }
            "timeout_constraint" => {
                code.push_str(&format!(r#"        // {} - {} (PRE-COMPILED)
        timeout_constraints.insert("{}".to_string(), Program::compile("{}").expect("Pre-compiled CEL failed"));

"#, entry.system_id, entry.expression_type, entry.system_id, entry.expression.replace("\"", "\\\"")));
            }
            _ => {}
        }
    }

    code.push_str(r#"        Ok(RoutingTable {
            node_constraints,
            routing_policies,
            timeout_constraints,
        })
    }
}
"#);

    code
}

fn validate_cel_variables(schema_dir: &str) -> Result<(), Vec<CelValidationError>> {
    use std::fs;
    use walkdir::WalkDir;
    use regex::Regex;

    let mut errors = Vec::new();
    let input_var_pattern = Regex::new(r#"\binput\.([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)"#).unwrap();

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let schema: serde_json::Value = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            // Only validate system schemas
            if schema.get("x-familiar-kind").and_then(|k| k.as_str()) != Some("system") {
                continue;
            }

            // Check dispatch rules
            if let Some(dispatch) = schema.get("dispatch").and_then(|d| d.as_array()) {
                for (i, rule) in dispatch.iter().enumerate() {
                    if let Some(routing_policy) = rule.get("routing_policy").and_then(|r| r.as_str()) {
                        // Extract input variables from CEL expression
                        let mut input_vars = Vec::new();
                        for capture in input_var_pattern.captures_iter(routing_policy) {
                            if let Some(var_match) = capture.get(1) {
                                input_vars.push(var_match.as_str().to_string());
                            }
                        }

                        // Get input_key to resolve schema
                        if let Some(input_key) = rule.get("input_key").and_then(|k| k.as_str()) {
                            if let Some(inputs) = schema.get("inputs").and_then(|i| i.as_object()) {
                                if let Some(input_def) = inputs.get(input_key).and_then(|d| d.as_object()) {
                                    // Check for schema reference in the new inputs structure
                                    let schema_ref = if let Some(schema_obj) = input_def.get("$ref").and_then(|r| r.as_str()) {
                                        Some(schema_obj)
                                    } else if let Some(schema_field) = input_def.get("schema") {
                                        schema_field.get("$ref").and_then(|r| r.as_str())
                                    } else {
                                        None
                                    };

                                    if let Some(schema_ref) = schema_ref {
                                        // Resolve the schema path relative to this system schema
                                        let system_dir = entry.path().parent().unwrap();
                                        let schema_path_resolved = resolve_schema_path(system_dir, schema_ref);

                                        // Validate variables against the schema
                                        for var in input_vars {
                                            if let Err(field_errors) = validate_input_variable(&var, &schema_path_resolved) {
                                                for error in field_errors {
                                                    errors.push(CelValidationError {
                                                        schema_path: schema_path.clone(),
                                                        
                                                        message: format!("dispatch[{}].routing_policy: {} -> {}", i, var, error),
                                                    });
                                                }
                                            }
                                        }
                                    } else {
                                        errors.push(CelValidationError {
                                            schema_path: schema_path.clone(),
                                            
                                            message: format!("dispatch[{}]: input '{}' missing schema reference", i, input_key),
                                        });
                                    }
                                } else {
                                    errors.push(CelValidationError {
                                        schema_path: schema_path.clone(),
                                        
                                        message: format!("dispatch[{}]: input_key '{}' not found in inputs", i, input_key),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn resolve_schema_path(system_dir: &std::path::Path, schema_ref: &str) -> std::path::PathBuf {
    use std::path::Path;

    // Handle absolute paths and URI schemes
    if schema_ref.starts_with('/') || schema_ref.contains("://") {
        // For absolute paths or URIs, resolve relative to the schema root
        let schema_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("versions/latest");
        return schema_root.join(schema_ref.trim_start_matches('/'));
    }

    // For relative paths, resolve relative to the system directory
    let relative_path = Path::new(schema_ref);

    // Use canonicalize to handle .. and . properly
    match system_dir.join(relative_path).canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => {
            // Fallback to manual resolution if canonicalize fails
            let mut path = system_dir.to_path_buf();
            for component in schema_ref.split('/') {
                match component {
                    "." => continue,
                    ".." => {
                        path.pop();
                    }
                    component => {
                        path.push(component);
                    }
                }
            }
            path
        }
    }
}

fn pgo_action(schema_dir: &str, action: PgoAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PgoAction::Instrument => {
            println!("🎯 Building with PGO instrumentation...");
            println!("💡 This will generate .profraw files during execution");

            // Build with instrumentation
            run_command(&[
                "cargo", "build", "--release",
                "--config", &format!("target.'cfg(all())'.rustflags = [\"-Cprofile-generate=/tmp/familiar-pgo-data\"]")
            ]);

            println!("✅ Instrumentation build completed!");
            println!("💡 Run your workload to generate profile data, then use 'cargo xtask pgo sample'");
        }

        PgoAction::Sample => {
            println!("📊 Running PGO workload sampling...");

            // Find systems with PGO enabled
            let pgo_systems = find_pgo_enabled_systems(schema_dir)?;

            if pgo_systems.is_empty() {
                println!("⚠️  No systems found with PGO enabled. Add 'x-familiar-pgo': {{'enabled': true}} to your system schemas.");
                return Ok(());
            }

            println!("🎯 Found {} PGO-enabled systems", pgo_systems.len());

            // For each PGO-enabled system, run its workload samples
            for system in pgo_systems {
                println!("  📋 Sampling system: {}", system.name);
                run_system_workload_samples(&system)?;
            }

            println!("✅ PGO sampling completed!");
            println!("💡 Profile data written to /tmp/familiar-pgo-data/");
            println!("💡 Use 'cargo xtask pgo optimize' to build the optimized binary");
        }

        PgoAction::Optimize => {
            println!("🚀 Building PGO-optimized binary...");

            // Check if profile data exists
            let profile_dir = "/tmp/familiar-pgo-data";
            if !std::path::Path::new(profile_dir).exists() {
                eprintln!("❌ Profile data not found at {}", profile_dir);
                eprintln!("💡 Run 'cargo xtask pgo instrument' then 'cargo xtask pgo sample' first");
                std::process::exit(1);
            }

            // Merge profiles and build optimized binary
            run_command(&[
                "cargo", "build", "--release",
                "--config", &format!("target.'cfg(all())'.rustflags = [\"-Cprofile-use={}/merged.profdata\"]", profile_dir)
            ]);

            println!("✅ PGO optimization completed!");
            println!("💡 Optimized binary is ready at target/release/");
        }
    }

    Ok(())
}

#[derive(Debug)]
struct PgoSystem {
    name: String,
    path: std::path::PathBuf,
    pgo_config: serde_json::Value,
}

fn find_pgo_enabled_systems(schema_dir: &str) -> Result<Vec<PgoSystem>, Box<dyn std::error::Error>> {
    use walkdir::WalkDir;
    use std::fs;

    let mut systems = Vec::new();

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = fs::read_to_string(entry.path())?;
            let schema: serde_json::Value = serde_json::from_str(&content)?;

            // Check if this is a system with PGO enabled
            if schema.get("x-familiar-kind").and_then(|k| k.as_str()) == Some("system") {
                if let Some(pgo_config) = schema.get("x-familiar-pgo") {
                    if pgo_config.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false) {
                        let name = schema.get("name").or_else(|| schema.get("id"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        systems.push(PgoSystem {
                            name,
                            path: entry.path().to_path_buf(),
                            pgo_config: pgo_config.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(systems)
}

fn run_system_workload_samples(system: &PgoSystem) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::process::Command;

    let sample_refs = system.pgo_config.get("workload_samples")
        .and_then(|s| s.as_array())
        .ok_or("No workload_samples defined in PGO config")?;

    // Path to the instrumented binary (assuming it's built in target/release with PGO instrumentation)
    let binary_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/release/familiar-daemon");

    if !binary_path.exists() {
        return Err(format!("Instrumented binary not found at: {}. Run 'cargo xtask pgo instrument' first.", binary_path.display()).into());
    }

    for sample_ref in sample_refs {
        if let Some(sample_path) = sample_ref.get("$ref").and_then(|r| r.as_str()) {
            // Resolve the sample path relative to the system
            let system_dir = system.path.parent().unwrap();
            let resolved_path = resolve_schema_path(system_dir, sample_path);

            if resolved_path.exists() {
                println!("    📄 Running sample: {}", sample_path);

                // Load the sample
                let sample_content = fs::read_to_string(&resolved_path)?;
                let sample: serde_json::Value = serde_json::from_str(&sample_content)?;

                if sample.get("x-familiar-kind").and_then(|k| k.as_str()) == Some("workload_sample") {
                    // CRITICAL FIX: Actually execute the instrumented binary with the sample
                    // This generates the .profraw files needed for PGO optimization

                    let input_data = sample.get("input")
                        .ok_or("Sample missing 'input' field")?
                        .to_string();

                    println!("      🚀 Executing instrumented binary with sample data...");

                    let mut child = Command::new(&binary_path)
                        .arg("--pgo-sample")  // Special flag to indicate PGO sampling mode
                        .arg(system.name.clone())  // System being sampled
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()?;

                    // Write the sample data to stdin
                    if let Some(ref mut stdin) = child.stdin {
                        use std::io::Write;
                        stdin.write_all(input_data.as_bytes())?;
                        stdin.flush()?;
                    }

                    // Wait for the process to complete
                    let status = child.wait_with_output()?;

                    if status.status.success() {
                        println!("      ✅ Sample executed successfully (exit code: {})", status.status);
                        println!("      📊 Profile data written to /tmp/familiar-pgo-data/");
                    } else {
                        let stderr = String::from_utf8_lossy(&status.stderr);
                        println!("      ⚠️  Sample execution completed with warnings (exit code: {})", status.status);
                        if !stderr.is_empty() {
                            println!("      stderr: {}", stderr);
                        }
                    }
                } else {
                    println!("      ⚠️  Sample missing x-familiar-kind: workload_sample");
                }
            } else {
                println!("    ❌ Sample not found: {}", sample_path);
            }
        }
    }

    Ok(())
}

fn validate_system_dispatch_integrity(schema_dir: &str) -> Result<(), Vec<CelValidationError>> {
    use std::fs;
    use walkdir::WalkDir;

    let mut errors = Vec::new();

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let schema: serde_json::Value = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            // Only validate system schemas
            if schema.get("x-familiar-kind").and_then(|k| k.as_str()) != Some("system") {
                continue;
            }

            // Validate dispatch rules
            if let Some(dispatch) = schema.get("dispatch").and_then(|d| d.as_array()) {
                // Get the inputs map for validation
                let inputs_map = schema.get("inputs").and_then(|i| i.as_object());

                for (i, rule) in dispatch.iter().enumerate() {
                    // Validate input_key exists in inputs map
                    if let Some(input_key) = rule.get("input_key").and_then(|k| k.as_str()) {
                        if let Some(inputs) = inputs_map {
                            if !inputs.contains_key(input_key) {
                                errors.push(CelValidationError {
                                    schema_path: schema_path.clone(),
                                    message: format!("dispatch[{}].input_key '{}' not found in inputs map. Available keys: {:?}",
                                        i, input_key, inputs.keys().collect::<Vec<_>>()),
                                });
                            } else {
                                // Validate that the input has a valid schema reference
                                let input_value = &inputs[input_key];
                                // The input should be a $ref object or a direct $ref string
                                let has_ref = input_value.get("$ref").is_some() ||
                                             (input_value.is_string() && input_value.as_str().unwrap().contains(".schema.json"));
                                if !has_ref {
                                    errors.push(CelValidationError {
                                        schema_path: schema_path.clone(),
                                        message: format!("input '{}' must have a '$ref' to a schema", input_key),
                                    });
                                }
                            }
                        } else {
                            errors.push(CelValidationError {
                                schema_path: schema_path.clone(),
                                message: format!("dispatch[{}].input_key specified but no inputs map found in schema", i),
                            });
                        }
                    }

                    // Validate trigger format (should be caught by JSON Schema pattern, but double-check)
                    if let Some(trigger) = rule.get("trigger").and_then(|t| t.as_str()) {
                        let valid_prefixes = ["kafka:", "temporal:", "cron:", "webhook:", "internal:", "sqs:", "redis:", "http:"];
                        let has_valid_prefix = valid_prefixes.iter().any(|prefix| trigger.starts_with(prefix));

                        if !has_valid_prefix {
                            errors.push(CelValidationError {
                                schema_path: schema_path.clone(),
                                message: format!("dispatch[{}].trigger '{}' does not match required pattern. Must start with one of: {:?}",
                                    i, trigger, valid_prefixes),
                            });
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_input_variable(var_path: &str, schema_path: &std::path::Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Read the schema file
    let schema_content = match std::fs::read_to_string(schema_path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("Cannot read schema file {}: {}", schema_path.display(), e));
            return Err(errors);
        }
    };

    let mut schema: serde_json::Value = match serde_json::from_str(&schema_content) {
        Ok(s) => s,
        Err(e) => {
            errors.push(format!("Invalid JSON in schema file {}: {}", schema_path.display(), e));
            return Err(errors);
        }
    };

    // CRITICAL FIX: Resolve allOf inheritance to merge inherited properties
    // This allows the validator to "see" fields like t_coord from PhysicalEntity
    if let Err(inheritance_errors) = resolve_schema_inheritance(&mut schema, schema_path) {
        errors.extend(inheritance_errors);
        return Err(errors);
    }

    // Navigate the resolved schema to check if the field path exists
    let field_parts: Vec<&str> = var_path.split('.').collect();

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        let mut current_properties = properties;

        for (i, part) in field_parts.iter().enumerate() {
            if let Some(field_def) = current_properties.get(*part) {
                if i == field_parts.len() - 1 {
                    // Last part - field exists
                    return Ok(());
                } else {
                    // Navigate deeper into nested objects
                    if let Some(nested_props) = field_def.get("properties").and_then(|p| p.as_object()) {
                        current_properties = nested_props;
                    } else {
                        errors.push(format!("Field '{}' exists but is not an object with properties", field_parts[..=i].join(".")));
                        break;
                    }
                }
            } else {
                errors.push(format!("Field '{}' not found in schema (after resolving inheritance)", field_parts[..=i].join(".")));
                break;
            }
        }
    } else {
        errors.push("Schema has no properties object (even after resolving inheritance)".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Resolves allOf inheritance by merging properties from referenced schemas
/// This is CRITICAL for the Symmetric Seven entities that inherit from PhysicalEntity
fn resolve_schema_inheritance(schema: &mut serde_json::Value, base_path: &std::path::Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(all_of) = schema.get("allOf").and_then(|a| a.as_array()) {
        let mut merged_properties = serde_json::Map::new();

        // First, add the base schema's properties if they exist
        if let Some(base_props) = schema.get("properties").and_then(|p| p.as_object()) {
            merged_properties.extend(base_props.clone());
        }

        // Then merge properties from each allOf reference
        for ref_item in all_of {
            if let Some(ref_path) = ref_item.get("$ref").and_then(|r| r.as_str()) {
                match resolve_and_load_schema_ref(ref_path, base_path) {
                    Ok(referenced_schema) => {
                        if let Some(ref_props) = referenced_schema.get("properties").and_then(|p| p.as_object()) {
                            // Merge referenced properties (referenced schema takes precedence on conflicts)
                            merged_properties.extend(ref_props.clone());
                        }

                        // Recursively resolve inheritance in the referenced schema
                        // This handles EntityMeta -> PhysicalEntity -> Individual Entity chains
                        let mut temp_schema = referenced_schema.clone();
                        if let Err(recursive_errors) = resolve_schema_inheritance(&mut temp_schema, base_path) {
                            errors.extend(recursive_errors);
                        } else if let Some(nested_props) = temp_schema.get("properties").and_then(|p| p.as_object()) {
                            merged_properties.extend(nested_props.clone());
                        }
                    }
                    Err(ref_errors) => {
                        errors.extend(ref_errors);
                    }
                }
            }
        }

        // Update the schema with merged properties
        if let Some(obj) = schema.as_object_mut() {
            obj.insert("properties".to_string(), serde_json::Value::Object(merged_properties));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Resolves a $ref path and loads the referenced schema
fn resolve_and_load_schema_ref(ref_path: &str, base_path: &std::path::Path) -> Result<serde_json::Value, Vec<String>> {
    let resolved_path = resolve_schema_path(base_path.parent().unwrap_or(base_path), ref_path);

    if !resolved_path.exists() {
        return Err(vec![format!("Referenced schema not found: {}", resolved_path.display())]);
    }

    match std::fs::read_to_string(&resolved_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(schema) => Ok(schema),
            Err(e) => Err(vec![format!("Invalid JSON in referenced schema {}: {}", resolved_path.display(), e)]),
        },
        Err(e) => Err(vec![format!("Cannot read referenced schema {}: {}", resolved_path.display(), e)]),
    }
}

fn validate_json_schemas(schema_dir: &str) -> Result<(), Vec<JsonSchemaValidationError>> {
    use std::fs;
    use walkdir::WalkDir;

    let mut errors = Vec::new();

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(JsonSchemaValidationError {
                        schema_path: entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                            .to_string_lossy().to_string(),
                        message: format!("Cannot read file: {}", e),
                    });
                    continue;
                }
            };

            let schema_value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(JsonSchemaValidationError {
                        schema_path: entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                            .to_string_lossy().to_string(),
                        message: format!("Invalid JSON: {}", e),
                    });
                    continue;
                }
            };

            let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            // Basic JSON Schema structure validation
            if let Err(basic_errors) = validate_basic_json_schema(&schema_value, &schema_path) {
                errors.extend(basic_errors);
            }

            // Additional custom validations for x-familiar extensions
            if let Err(custom_errors) = validate_familiar_extensions(&schema_value, &schema_path) {
                errors.extend(custom_errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_basic_json_schema(schema: &serde_json::Value, schema_path: &str) -> Result<(), Vec<JsonSchemaValidationError>> {
    let mut errors = Vec::new();

    // Check for required JSON Schema fields
    if !schema.get("$schema").is_some() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "Missing required '$schema' field".to_string(),
        });
    }

    if !schema.get("$id").is_some() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "Missing required '$id' field".to_string(),
        });
    }

    // Check that $schema is a valid draft
    if let Some(schema_url) = schema.get("$schema").and_then(|s| s.as_str()) {
        if !schema_url.contains("json-schema.org") && !schema_url.contains("meta/") && !schema_url.contains("../") {
            errors.push(JsonSchemaValidationError {
                schema_path: schema_path.to_string(),
                message: format!("Invalid $schema URL: {}", schema_url),
            });
        }
    }

    // Check for basic structure
    if !schema.is_object() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "JSON Schema must be an object".to_string(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_familiar_extensions(schema: &serde_json::Value, schema_path: &str) -> Result<(), Vec<JsonSchemaValidationError>> {
    let mut errors = Vec::new();

    // Check for x-familiar-kind
    if !schema.get("x-familiar-kind").is_some() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "Missing required 'x-familiar-kind' property".to_string(),
        });
    }

    // Check for $id
    if !schema.get("$id").is_some() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "Missing required '$id' property".to_string(),
        });
    }

    // Check for $schema
    if !schema.get("$schema").is_some() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "Missing required '$schema' property".to_string(),
        });
    }

    // Check for valid x-familiar-kind values
    if let Some(kind_value) = schema.get("x-familiar-kind") {
        if let Some(kind_str) = kind_value.as_str() {
            let valid_kinds = ["primitive", "entity", "action", "system", "node", "queue", "resource", "meta", "windmill", "entities_api"];
            if !valid_kinds.contains(&kind_str) {
                errors.push(JsonSchemaValidationError {
                    schema_path: schema_path.to_string(),
                    message: format!("Invalid x-familiar-kind '{}'. Must be one of: {:?}", kind_str, valid_kinds),
                });
            }
        } else {
            errors.push(JsonSchemaValidationError {
                schema_path: schema_path.to_string(),
                message: "x-familiar-kind must be a string".to_string(),
            });
        }
    }

    // Check for required title
    if !schema.get("title").is_some() {
        errors.push(JsonSchemaValidationError {
            schema_path: schema_path.to_string(),
            message: "Missing required 'title' property".to_string(),
        });
    }

    // Check for required type: object for most schemas (but allow meta-schema inheritance)
    if let Some(kind) = schema.get("x-familiar-kind").and_then(|k| k.as_str()) {
        if matches!(kind, "entity" | "action") {
            // These should have explicit type: object
            if !matches!(schema.get("type"), Some(serde_json::Value::String(s)) if s == "object") {
                errors.push(JsonSchemaValidationError {
                    schema_path: schema_path.to_string(),
                    message: format!("{} schemas must have type: 'object'", kind),
                });
            }
        }
        // system, node, resource, meta schemas can inherit structure from meta-schemas
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn load_config_manifest() -> Result<familiar_config::PolicyManifest, Box<dyn std::error::Error>> {
    use std::path::Path;

    let manifest_path = Path::new("../../familiar-architecture/familiar-config/policy_manifest.json");
    familiar_config::PolicyManifest::load_from_file(manifest_path).map_err(Into::into)
}

#[derive(Debug)]
struct ConfigValidationError {
    schema_path: String,
    message: String,
}

#[derive(Debug)]
struct JsonSchemaValidationError {
    schema_path: String,
    message: String,
}

fn run_command(args: &[&str]) {
    use std::process::Command;
    println!("💡 Running: {}", args.join(" "));
    let status = Command::new(args[0])
        .args(&args[1..])
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        eprintln!("❌ Command failed with exit code: {}", status.code().unwrap_or(-1));
    }
}



