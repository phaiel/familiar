//! Protobuf Codegen CLI
//!
//! Manages Protobuf schemas for Kafka messaging:
//! 1. Exports schemas to files for version control
//! 2. Prints schemas for inspection
//!
//! Note: Schema Registry operations have been removed in favor of the
//! Opaque Envelope pattern where the envelope is static Protobuf and
//! payloads are opaque JSON bytes.
//!
//! # Usage
//!
//! ```bash
//! # Print schemas
//! cargo run --bin protobuf_codegen --features protobuf -- --print
//!
//! # Export schemas to files
//! cargo run --bin protobuf_codegen --features protobuf -- --export ./schemas
//! ```

use clap::Parser;
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "protobuf_codegen")]
#[command(about = "Generate and register Protobuf schemas for Kafka messaging")]
struct Args {
    /// Export schemas to directory
    #[arg(long)]
    export: Option<PathBuf>,
    
    /// Print schemas to stdout
    #[arg(long)]
    print: bool,
    
    /// Path to proto directory
    #[arg(long, default_value = "proto")]
    proto_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Proto files to process
    let proto_files = vec![
        ("payload", "familiar.kafka.Payload", "payload.proto"),
        ("envelope_v1", "familiar.envelope.v1-value", "envelope_v1.proto"),
    ];
    
    // Print schemas
    if args.print {
        println!("=== Protobuf Schemas ===\n");
        for (name, subject, filename) in &proto_files {
            println!("--- {} ({}) ---", name, subject);
            let filepath = args.proto_dir.join(filename);
            if filepath.exists() {
                let content = fs::read_to_string(&filepath)?;
                println!("{}", content);
            } else {
                println!("(File not found: {:?})", filepath);
            }
            println!();
        }
    }
    
    // Export schemas
    if let Some(export_dir) = &args.export {
        println!("Exporting schemas to {:?}...", export_dir);
        fs::create_dir_all(export_dir)?;
        
        for (_name, _subject, filename) in &proto_files {
            let src = args.proto_dir.join(filename);
            let dst = export_dir.join(filename);
            
            if src.exists() {
                fs::copy(&src, &dst)?;
                println!("  Exported: {}", dst.display());
            } else {
                println!("  Skipped (not found): {:?}", src);
            }
        }
    }
    
    // If no action specified, print help
    if args.export.is_none() && !args.print {
        println!("No action specified. Use --help for usage information.");
        println!("\nQuick start:");
        println!("  # Print schemas");
        println!("  cargo run --bin protobuf_codegen --features protobuf -- --print");
        println!();
        println!("  # Export schemas to files");
        println!("  cargo run --bin protobuf_codegen --features protobuf -- --export ./schemas");
    }
    
    Ok(())
}
