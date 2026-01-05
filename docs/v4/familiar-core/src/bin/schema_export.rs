//! Schema Export CLI
//!
//! Exports Rust types to JSON Schema format using schemars.
//! This tool syncs familiar-schemas to match current Rust types.
//!
//! Usage: cargo run --bin schema_export -- --output-dir /path/to/output

use clap::Parser;
use ::config::{Config, Environment, File};
use familiar_core::config::schema_lock;
use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// Import types from familiar-core for export
use familiar_core::entities::*;
use familiar_core::components::*;
use familiar_core::primitives::*;
use familiar_core::types::auth::*;
use familiar_core::types::api::*;

/// Configuration for the schema export
#[derive(Debug, Deserialize, Default)]
struct ExportConfig {
    /// Default output directory for schemas
    #[serde(default)]
    output_dir: Option<String>,
    
    /// Schema version to generate (e.g., "v0.8.0")
    #[serde(default = "default_version")]
    schema_version: String,
}

fn default_version() -> String {
    "v0.8.0".to_string()
}

static CONFIG: OnceLock<ExportConfig> = OnceLock::new();

fn load_config() -> ExportConfig {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    
    Config::builder()
        .set_default("schema_version", "v0.8.0").unwrap_or_default()
        .add_source(File::with_name(&format!("{}/config", manifest_dir)).required(false))
        .add_source(File::with_name(&format!("{}/../config", manifest_dir)).required(false))
        .add_source(Environment::with_prefix("FAMILIAR").separator("_"))
        .build()
        .and_then(|c| c.try_deserialize())
        .unwrap_or_default()
}

fn get_config() -> &'static ExportConfig {
    CONFIG.get_or_init(load_config)
}

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "schema_export")]
#[command(about = "Export Rust types to JSON Schema format")]
struct Args {
    /// Output directory for schemas (uses schema.lock [source] configuration by default)
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
    
    /// Dry run - print what would be exported without writing files
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();
    let config = get_config();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    
    // Determine output directory using schema.lock configuration
    let output_dir = args.output_dir.unwrap_or_else(|| {
        if let Some(ref dir) = config.output_dir {
            PathBuf::from(dir)
        } else {
            schema_lock::find_schema_dir(&PathBuf::from(manifest_dir))
                .unwrap_or_else(|| PathBuf::from(manifest_dir).join("schemas"))
        }
    });
    
    println!("--- SCHEMA EXPORT ---");
    println!();
    println!("Output directory: {}", output_dir.display());
    println!("Schema version: {}", config.schema_version);
    if args.dry_run {
        println!("DRY RUN - no files will be written");
    }
    println!();
    
    let mut exported = 0;
    
    // Export core entities
    exported += export::<Bond>(&output_dir, "entities", args.dry_run);
    exported += export::<Course>(&output_dir, "entities", args.dry_run);
    exported += export::<Filament>(&output_dir, "entities", args.dry_run);
    exported += export::<Focus>(&output_dir, "entities", args.dry_run);
    exported += export::<Intent>(&output_dir, "entities", args.dry_run);
    exported += export::<Moment>(&output_dir, "entities", args.dry_run);
    exported += export::<Motif>(&output_dir, "entities", args.dry_run);
    exported += export::<Pulse>(&output_dir, "entities", args.dry_run);
    exported += export::<Shuttle>(&output_dir, "entities", args.dry_run);
    exported += export::<Thread>(&output_dir, "entities", args.dry_run);
    
    // Export core components
    exported += export::<BondPhysics>(&output_dir, "components", args.dry_run);
    exported += export::<ChatMessage>(&output_dir, "components", args.dry_run);
    exported += export::<ClassificationSuperposition>(&output_dir, "components", args.dry_run);
    exported += export::<CognitiveDimensions>(&output_dir, "components", args.dry_run);
    exported += export::<CognitiveOptics>(&output_dir, "components", args.dry_run);
    exported += export::<ContentPayload>(&output_dir, "components", args.dry_run);
    exported += export::<DbPoolConfig>(&output_dir, "components", args.dry_run);
    exported += export::<EmotionalState>(&output_dir, "components", args.dry_run);
    exported += export::<FieldExcitation>(&output_dir, "components", args.dry_run);
    exported += export::<Identity>(&output_dir, "components", args.dry_run);
    exported += export::<LlmRequestDebug>(&output_dir, "components", args.dry_run);
    exported += export::<Metadata>(&output_dir, "components", args.dry_run);
    exported += export::<ObservationRequest>(&output_dir, "components", args.dry_run);
    exported += export::<ObservationResponse>(&output_dir, "components", args.dry_run);
    exported += export::<PhysicsHint>(&output_dir, "components", args.dry_run);
    exported += export::<ProviderConfig>(&output_dir, "components", args.dry_run);
    exported += export::<QuantumState>(&output_dir, "components", args.dry_run);
    exported += export::<RelationalDynamics>(&output_dir, "components", args.dry_run);
    exported += export::<RequestConfig>(&output_dir, "components", args.dry_run);
    exported += export::<ResponseMetadata>(&output_dir, "components", args.dry_run);
    exported += export::<SimLOD>(&output_dir, "components", args.dry_run);
    exported += export::<SimulationTier>(&output_dir, "components", args.dry_run);
    exported += export::<TaskDynamics>(&output_dir, "components", args.dry_run);
    exported += export::<Timestamps>(&output_dir, "components", args.dry_run);
    exported += export::<Weave>(&output_dir, "components", args.dry_run);
    exported += export::<WeaveUnit>(&output_dir, "components", args.dry_run);
    exported += export::<WeaveUnitClassification>(&output_dir, "components", args.dry_run);
    exported += export::<WeightedClassification>(&output_dir, "components", args.dry_run);

    // Export primitives
    exported += export::<UserId>(&output_dir, "primitives", args.dry_run);
    exported += export::<TenantId>(&output_dir, "primitives", args.dry_run);
    exported += export::<ChannelId>(&output_dir, "primitives", args.dry_run);
    exported += export::<MessageId>(&output_dir, "primitives", args.dry_run);
    exported += export::<EntityId>(&output_dir, "primitives", args.dry_run);
    exported += export::<TaskId>(&output_dir, "primitives", args.dry_run);
    exported += export::<CourseId>(&output_dir, "primitives", args.dry_run);
    exported += export::<ShuttleId>(&output_dir, "primitives", args.dry_run);
    exported += export::<ThreadId>(&output_dir, "primitives", args.dry_run);
    exported += export::<SessionId>(&output_dir, "primitives", args.dry_run);
    exported += export::<InvitationId>(&output_dir, "primitives", args.dry_run);
    exported += export::<JoinRequestId>(&output_dir, "primitives", args.dry_run);
    exported += export::<MagicLinkId>(&output_dir, "primitives", args.dry_run);
    exported += export::<AuditLogId>(&output_dir, "primitives", args.dry_run);
    exported += export::<Email>(&output_dir, "primitives", args.dry_run);
    exported += export::<PasswordHash>(&output_dir, "primitives", args.dry_run);
    exported += export::<NormalizedFloat>(&output_dir, "primitives", args.dry_run);
    exported += export::<Temperature>(&output_dir, "primitives", args.dry_run);
    exported += export::<MaxTokens>(&output_dir, "primitives", args.dry_run);
    exported += export::<Timestamp>(&output_dir, "primitives", args.dry_run);
    
    println!("Finished! Exported {} schemas.", exported);
}

fn export<T: JsonSchema>(output_dir: &Path, category: &str, dry_run: bool) -> usize {
    let name = std::any::type_name::<T>().split("::").last().unwrap();
    let schema = schemars::schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    
    let target_dir = output_dir.join(category);
    let target_filename = format!("{}.schema.json", name);
    let target_path = target_dir.join(target_filename);
    
    if !dry_run {
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(&target_path, json).unwrap();
        println!("✅ Exported {}/{}", category, name);
    } else {
        println!("🔍 Would export {}/{}", category, name);
    }
    
    1
}
