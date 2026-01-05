//! Drift Report CLI
//!
//! Generates a report of drift between Rust types and JSON schemas.
//! This tool helps identify types that are eligible for generation
//! and those that require manual maintenance.

use clap::Parser;
use ::config::{Config, Environment, File};
use familiar_core::config::schema_lock;
use familiar_core::schemas::SCHEMA_VERSION;
use familiar_drift_internals::parser;
use familiar_drift_internals::schema;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use walkdir::WalkDir;

/// Configuration for the drift report
#[derive(Debug, Deserialize, Default)]
struct ReportConfig {
    /// Schema version to check (e.g., "v0.8.0")
    #[serde(default = "default_version")]
    schema_version: String,
}

fn default_version() -> String {
    "v0.8.0".to_string()
}

static CONFIG: OnceLock<ReportConfig> = OnceLock::new();

fn load_config() -> ReportConfig {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    
    Config::builder()
        .set_default("schema_version", SCHEMA_VERSION).unwrap_or_default()
        .add_source(File::with_name(&format!("{}/config", manifest_dir)).required(false))
        .add_source(File::with_name(&format!("{}/../config", manifest_dir)).required(false))
        .add_source(Environment::with_prefix("FAMILIAR").separator("_"))
        .build()
        .and_then(|c| c.try_deserialize())
        .unwrap_or_default()
}

fn get_config() -> &'static ReportConfig {
    CONFIG.get_or_init(load_config)
}

#[derive(Parser, Debug)]
#[command(name = "drift_report")]
#[command(about = "Generate a drift report between Rust types and JSON schemas")]
struct Args {
    /// Path to schemas directory
    #[arg(short, long)]
    schemas_dir: Option<PathBuf>,
}

#[derive(Debug)]
struct SchemaInfo {
    name: String,
    path: PathBuf,
    is_simple: bool,
}

#[derive(Debug)]
struct RustInfo {
    name: String,
    path: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let config = get_config();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    
    // Resolve schemas directory using schema.lock or CLI arg
    let schemas_dir = args.schemas_dir.unwrap_or_else(|| {
        schema_lock::find_schema_dir(&PathBuf::from(manifest_dir))
            .unwrap_or_else(|| PathBuf::from(manifest_dir).join("schemas"))
    });
    
    if !schemas_dir.exists() {
        anyhow::bail!("Schema directory not found: {}", schemas_dir.display());
    }
    
    // Resolve Rust search directories
    let search_dirs = vec![
        PathBuf::from(manifest_dir).join("src/types"),
        PathBuf::from(manifest_dir).join("src/entities"),
        PathBuf::from(manifest_dir).join("src/components"),
        PathBuf::from(manifest_dir).join("src/primitives"),
    ];
    
    println!("+------------------------------------------------------------------+");
    println!("|                      SCHEMA DRIFT REPORT                         |");
    println!("+------------------------------------------------------------------+");
    println!();
    println!("Schema version: {}", config.schema_version);
    println!("Schema path: {}", schemas_dir.display());
    println!();
    
    let (rust_types, rust_type_paths) = collect_rust_types(&search_dirs)?;
    let (schemas, schema_paths) = collect_json_schemas(&schemas_dir)?;
    
    println!("Found {} JSON schemas", schemas.len());
    println!("Found {} Rust types", rust_types.len());
    
    // Find mismatches
    let rust_type_names: HashSet<String> = rust_types.keys().cloned().collect();
    let schema_names: HashSet<String> = schemas.keys().cloned().collect();
    
    let rust_only: Vec<_> = rust_type_names.difference(&schema_names).collect();
    let schema_only: Vec<_> = schema_names.difference(&rust_type_names).collect();
    let both: Vec<_> = rust_type_names.intersection(&schema_names).collect();
    
    if !rust_only.is_empty() {
        println!("\n+------------------------------------------------------------------+");
        println!("| TYPES IN RUST BUT NOT IN SCHEMAS ({} types)                  ", rust_only.len());
        println!("+------------------------------------------------------------------+");
        for name in &rust_only {
            println!("|  {} ({})", name, rust_type_paths.get(*name).unwrap().display());
        }
        println!("+------------------------------------------------------------------+");
    }
    
    if !schema_only.is_empty() {
        println!("\n+------------------------------------------------------------------+");
        println!("| TYPES IN SCHEMAS BUT NOT IN RUST ({} types)                  ", schema_only.len());
        println!("+------------------------------------------------------------------+");
        for name in &schema_only {
            let info = schemas.get(*name).unwrap();
            let eligibility = if info.is_simple { "[simple]" } else { "[COMPLEX]" };
            println!("|  {} {} ({})", name, eligibility, schema_paths.get(*name).unwrap().display());
        }
        println!("+------------------------------------------------------------------+");
    }
    
    // Detailed field comparison for types in both
    let mut mismatches = Vec::new();
    for name in both {
        let schema_info = schema::find_and_parse_schema(&schemas_dir, name)?;
        let rust_info = parser::find_and_parse_rust_type_multi(
            &search_dirs.iter().map(|p| p.as_path()).collect::<Vec<_>>(),
            name
        )?;
        
        match familiar_drift_internals::compare::compare_types(name, &rust_info, &schema_info) {
            Ok(_) => {}
            Err(e) => mismatches.push((name, e, rust_info.has_flattened)),
        }
    }
    
    if !mismatches.is_empty() {
        println!("\n+------------------------------------------------------------------+");
        println!("| FIELD MISMATCHES ({} types with drift)                        ", mismatches.len());
        println!("+------------------------------------------------------------------+");
        for (name, error, flattened) in &mismatches {
            println!("|");
            println!("|  📦 {}", name);
            if *flattened {
                println!("|     (has #[serde(flatten)] - may have inherited fields)");
            }
            println!("|     {}", error);
        }
        println!("+------------------------------------------------------------------+");
    }
    
    // Calculate generation eligibility
    let simple_schemas: Vec<_> = schemas.values().filter(|s| s.is_simple).collect();
    let complex_schemas: Vec<_> = schemas.values().filter(|s| !s.is_simple).collect();
    
    println!("\n+------------------------------------------------------------------+");
    println!("| SCHEMA GENERATION ELIGIBILITY                                |");
    println!("+------------------------------------------------------------------+");
    println!("|  Simple (can generate):  {} schemas ({}%)", simple_schemas.len(), (simple_schemas.len() as f64 / schemas.len() as f64 * 100.0).round());
    println!("|  Complex (manual only): {} schemas ({}%)", complex_schemas.len(), (complex_schemas.len() as f64 / schemas.len() as f64 * 100.0).round());
    println!("+------------------------------------------------------------------+");
    
    println!("\n+------------------------------------------------------------------+");
    println!("| SUMMARY                                                      |");
    println!("+------------------------------------------------------------------+");
    println!("|  Total schemas:       {}", schemas.len());
    println!("|  Total Rust types:    {}", rust_types.len());
    println!("|  In both:             {}", rust_type_names.intersection(&schema_names).count());
    println!("|  Rust only:           {}", rust_only.len());
    println!("|  Schema only:         {}", schema_only.len());
    println!("|  Field mismatches:    {}", mismatches.len());
    println!("|");
    println!("|  Generation target:   {}% ({} types)", (simple_schemas.len() as f64 / schemas.len() as f64 * 100.0).round(), simple_schemas.len());
    println!("|  Manual maintenance:  {}% ({} types)", (complex_schemas.len() as f64 / schemas.len() as f64 * 100.0).round(), complex_schemas.len());
    println!("+------------------------------------------------------------------+");
    
    Ok(())
}

fn collect_rust_types(dirs: &[PathBuf]) -> Result<(HashMap<String, RustInfo>, HashMap<String, PathBuf>), anyhow::Error> {
    let mut types = HashMap::new();
    let mut paths = HashMap::new();
    
    for dir in dirs {
        if !dir.exists() { continue; }
        
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let content = fs::read_to_string(entry.path())?;
            // Simple heuristic to find struct/enum definitions
            for line in content.lines() {
                let line = line.trim();
                if (line.starts_with("pub struct ") || line.starts_with("pub enum ")) && !line.contains("(") {
                    let name = line.split_whitespace().nth(2)
                        .map(|s| s.split('{').next().unwrap().to_string());
                    if let Some(name) = name {
                        types.insert(name.clone(), RustInfo { name: name.clone(), path: entry.path().to_path_buf() });
                        paths.insert(name, entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    
    Ok((types, paths))
}

fn collect_json_schemas(dir: &Path) -> Result<(HashMap<String, SchemaInfo>, HashMap<String, PathBuf>), anyhow::Error> {
    let mut schemas = HashMap::new();
    let mut paths = HashMap::new();
    
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
    {
        let content = fs::read_to_string(entry.path())?;
        let _json: serde_json::Value = serde_json::from_str(&content)?;
        
        let name = entry.path().file_name().unwrap().to_string_lossy()
            .replace(".schema.json", "");
            
        let is_simple = !content.contains("anyOf") && !content.contains("oneOf") && !content.contains("allOf");
        
        schemas.insert(name.clone(), SchemaInfo { name: name.clone(), path: entry.path().to_path_buf(), is_simple });
        paths.insert(name, entry.path().to_path_buf());
    }
    
    Ok((schemas, paths))
}
