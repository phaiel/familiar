//! Familiar Schema MCP Server
//!
//! Model Context Protocol server exposing the schema graph to AI agents.
//!
//! The MCP uses schemas EMBEDDED at compile time from the locked version in schema.lock.
//! This ensures the MCP always serves the exact schemas that familiar-core was built with.
//!
//! ## Usage
//!
//! ```bash
//! # Start the server (typically called by Cursor via ~/.cursor/mcp.json)
//! familiar-mcp
//! ```
//!
//! ## Tools
//!
//! Meta: status, resolve, schema_raw
//! Query: get_type, get_refs, get_dependents, search, list_kinds
//! Agent: closure, imports_for, lint_unions, services_for_schema

use clap::Parser;
use familiar_core::mcp::{SchemaGraph, SchemaTools};
use familiar_core::schemas::generated_version::{SCHEMA_VERSION, SCHEMA_HASH};
use familiar_contracts::SCHEMAS;
use mcp_attr::server::serve_stdio;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "familiar-mcp")]
#[command(about = "MCP server for the Familiar schema registry")]
#[command(version)]
struct Args {
    /// Generate artifacts (schema_index.json) and exit
    #[arg(long)]
    generate_artifacts: bool,
    
    /// Output directory for artifacts
    #[arg(long, default_value = "artifacts")]
    artifacts_dir: PathBuf,
}

fn generate_artifacts(graph: &SchemaGraph, artifacts_dir: &PathBuf) -> anyhow::Result<()> {
    fs::create_dir_all(artifacts_dir)?;
    
    // Generate schema_index.json
    let index: Vec<serde_json::Value> = graph
        .all_schemas()
        .map(|node| {
            serde_json::json!({
                "id": node.id,
                "path": node.path,
                "title": node.title,
                "kind": node.kind,
                "service": node.service
            })
        })
        .collect();
    
    let index_path = artifacts_dir.join("schema_index.json");
    fs::write(
        &index_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "bundle_hash": graph.bundle_hash,
            "schema_count": graph.schema_count(),
            "edge_count": graph.edge_count(),
            "scc_count": graph.scc_count(),
            "schemas": index
        }))?
    )?;
    
    eprintln!("Generated: {}", index_path.display());
    
    // Generate kinds_index.json
    let kinds = graph.all_kinds();
    let kinds_index: serde_json::Value = kinds
        .into_iter()
        .map(|kind| {
            let schemas = graph.list_by_kind(&kind);
            (kind, serde_json::json!(schemas))
        })
        .collect();
    
    let kinds_path = artifacts_dir.join("kinds_index.json");
    fs::write(&kinds_path, serde_json::to_string_pretty(&kinds_index)?)?;
    eprintln!("Generated: {}", kinds_path.display());
    
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Load schema graph from familiar-contracts
    let start = Instant::now();
    let graph = SchemaGraph::from_embedded(&SCHEMAS)?;
    let load_time = start.elapsed();
    
    eprintln!(
        "familiar-mcp v{}: Loaded {} schemas, {} edges, {} SCCs in {:?}",
        env!("CARGO_PKG_VERSION"),
        graph.schema_count(),
        graph.edge_count(),
        graph.scc_count(),
        load_time
    );
    eprintln!("Schema version: {} (hash: {})", SCHEMA_VERSION, &SCHEMA_HASH[..20]);
    
    // Generate artifacts if requested
    if args.generate_artifacts {
        generate_artifacts(&graph, &args.artifacts_dir)?;
        return Ok(());
    }
    
    // Create tools handler
    let tools = SchemaTools::new(graph);
    
    eprintln!("MCP server ready, waiting for requests on stdio...");
    
    // Run MCP server
    serve_stdio(tools).await?;
    
    Ok(())
}

