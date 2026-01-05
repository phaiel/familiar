//! Minerva - The Master Weaver
//!
//! CLI-only entrypoint for the familiar-worker binary.
//!
//! # Usage
//!
//! ```bash
//! # Onboarding signup
//! minerva onboarding execute-signup --input '{"email": "...", "name": "..."}'
//!
//! # Evaluate signup (Evaluator Pattern)
//! minerva onboarding evaluate-signup --input '{"email": "...", "name": "..."}'
//!
//! # Fates pipeline
//! minerva fates pipeline --input '{"content": "...", ...}'
//!
//! # Health check
//! minerva maintenance health
//! ```
//!
//! # Environment Variables
//!
//! - `MINERVA_INPUT`: JSON input (alternative to --input)
//! - `MINERVA_VERBOSE`: Enable verbose logging
//! - `DATABASE_URL`: PostgreSQL connection string

use clap::Parser;
use tracing::{error, info};

use familiar_worker::cli::{Cli, Domain};
use familiar_worker::config::WorkerConfig;
use familiar_worker::evaluator::{EvaluationResult, EvaluationStep};
use familiar_worker::runtime::StepRuntime;

#[tokio::main]
async fn main() {
    // Load environment from .env file
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        "familiar_worker=debug,familiar_core=debug"
    } else {
        "familiar_worker=info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!(
        domain = cli.domain_name(),
        action = cli.action_name(),
        "Minerva awakening"
    );

    // Load configuration
    let config = match WorkerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            let error = EvaluationResult::new(
                EvaluationStep::Reject,
                format!("Configuration error: {}", e),
            );
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            std::process::exit(1);
        }
    };

    // Get input
    let input = match get_input(&cli) {
        Ok(i) => i,
        Err(e) => {
            let error = EvaluationResult::new(
                EvaluationStep::Reject,
                format!("Input error: {}", e),
            );
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            std::process::exit(1);
        }
    };

    // Execute
    let result = execute(&cli, config, &input).await;

    // Output result
    match result {
        Ok(output) => {
            println!("{}", output);
            info!("Minerva completed successfully");
        }
        Err(e) => {
            let error = EvaluationResult::new(
                EvaluationStep::Reject,
                e.to_string(),
            );
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            error!(error = %e, "Minerva encountered an error");
            std::process::exit(1);
        }
    }
}

/// Execute the requested command
async fn execute(
    cli: &Cli,
    config: WorkerConfig,
    input: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let runtime = StepRuntime::new(config).await?;

    let output = match &cli.domain {
        Domain::Fates { action } => runtime.execute_fates(action.clone(), input).await?,
        Domain::Onboarding { action } => runtime.execute_onboarding(action.clone(), input).await?,
        Domain::Manifold { action } => runtime.execute_manifold(action.clone(), input).await?,
        Domain::Maintenance { action } => runtime.execute_maintenance(action.clone(), input).await?,
    };

    Ok(output)
}

/// Get input JSON from CLI arguments or stdin
fn get_input(cli: &Cli) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Priority: --input > --input-file > stdin
    if let Some(ref input) = cli.input {
        return Ok(input.clone());
    }

    if let Some(ref path) = cli.input_file {
        return Ok(std::fs::read_to_string(path)?);
    }

    // Try to read from stdin
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    if buffer.is_empty() {
        // For commands that don't need input (like health check)
        return Ok("{}".to_string());
    }

    Ok(buffer)
}
