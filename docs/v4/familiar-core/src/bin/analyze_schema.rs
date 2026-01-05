//! Schema analyzer CLI - Fast Rust-based schema analysis
//!
//! A high-performance CLI tool for analyzing Rust schemas, TypeScript types,
//! and Python models. Uses ast-grep for single-file rules and Rust for
//! cross-file/manifest analysis.
//!
//! Performance optimizations:
//! - Built with LTO (link-time optimization)
//! - Uses ignore crate (ripgrep's file walker) for fast .gitignore-aware traversal
//! - Parallel analysis with rayon
//! - ast-grep for declarative pattern matching

use askama::Template;
use clap::{Parser, Subcommand};
use config::{Config as ConfigLoader, Environment, File};
use console::style;
use familiar_core::analysis::{AnalysisReport, AstGrepRunner, AutoFixer, CrossFileAnalyzer, DatabaseAnalyzer, IssueKind, OrphanRecommendation, Severity};
use familiar_core::reports::{HtmlReport, format_text_report};
// Progress bar support (for future use with verbose mode)
#[allow(unused_imports)]
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Fast schema analyzer for the Familiar codebase
#[derive(Parser, Debug)]
#[command(name = "analyze-schema")]
#[command(about = "Analyze Rust schemas, TypeScript types, and Python models for consistency")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze schemas (existing default behavior)
    #[command(name = "analyze")]
    Analyze(AnalyzeArgs),

    /// View and edit configuration in TUI
    #[command(name = "config")]
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
struct ConfigArgs {
    /// Path to config file (defaults to searching common locations)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Show secrets (WARNING: displays sensitive data)
    #[arg(long)]
    show_secrets: bool,

    /// Config service type (api, worker, core)
    #[arg(short, long)]
    service: Option<String>,
}

#[derive(Parser, Debug)]
struct AnalyzeArgs {
    /// Workspace root path
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Use ast-grep rules (fast, single-file patterns)
    #[arg(long)]
    ast_grep: bool,

    /// Use Rust analyzer (cross-file, manifest checks)
    #[arg(long)]
    rust: bool,

    /// Run both ast-grep and Rust analyzers (default)
    #[arg(long)]
    all: bool,

    /// Output report as JSON
    #[arg(long)]
    json: bool,

    /// Generate HTML report
    #[arg(long)]
    html: bool,

    /// Don't save reports to disk
    #[arg(long)]
    no_save: bool,

    /// Show detailed progress
    #[arg(short, long)]
    verbose: bool,

    /// Only show errors (suppress info/warnings)
    #[arg(short, long)]
    quiet: bool,

    /// Apply safe auto-fixes (non-breaking changes only)
    #[arg(long)]
    fix: bool,

    /// Apply all fixes including potentially breaking ones
    #[arg(long)]
    fix_unsafe: bool,

    /// Show what would be fixed without applying changes
    #[arg(long)]
    fix_dry_run: bool,

    /// Confirm unsafe fixes (required with --fix-unsafe)
    #[arg(long)]
    confirm: bool,

    /// Run Kafka/Protobuf codegen compliance checks
    #[arg(long)]
    kafka: bool,

    /// Include familiar-worker in analysis scope
    #[arg(long)]
    include_worker: bool,

    /// Run database/SeaORM compliance checks in services
    #[arg(long)]
    database: bool,

    /// Check for orphan schemas (schemas not connected in the graph)
    #[arg(long)]
    orphan_schemas: bool,

    /// Filter orphan analysis to a specific category (e.g., "tools", "auth")
    #[arg(long)]
    orphan_filter: Option<String>,
}

fn main() {
    let args = Args::parse();
    let start = Instant::now();

    match args.command {
        Commands::Analyze(analyze_args) => run_analyze(&analyze_args, start),
        Commands::Config(config_args) => load_config_viewer(&config_args),
    }
}

fn run_analyze(args: &AnalyzeArgs, start: Instant) {
    
    // Determine which analyzers to run
    let run_ast_grep = args.ast_grep || args.all || (!args.ast_grep && !args.rust);
    let run_rust = args.rust || args.all;
    
    if !args.quiet {
        println!("{}", style("🔍 Schema Analyzer").cyan().bold());
        println!("   Path: {}", args.path.display());
        let mode = if run_ast_grep && run_rust { "hybrid (ast-grep + Rust)" } 
            else if run_ast_grep { "ast-grep only" } 
            else { "Rust only" };
        let kafka_mode = if args.kafka { " + Kafka/Protobuf codegen" } else { "" };
        let worker_mode = if args.include_worker { " + familiar-worker" } else { "" };
        let database_mode = if args.database { " + Database/SeaORM" } else { "" };
        let orphan_mode = if args.orphan_schemas { 
            if let Some(ref filter) = args.orphan_filter {
                format!(" + Orphan Schemas ({})", filter)
            } else {
                " + Orphan Schemas".to_string()
            }
        } else { 
            String::new() 
        };
        println!("   Mode: {}{}{}{}{}", mode, kafka_mode, worker_mode, database_mode, orphan_mode);
        println!();
    }
    
    let mut combined_report: Option<AnalysisReport> = None;
    
    // Run ast-grep analysis
    if run_ast_grep {
        if args.verbose {
            println!("{}", style("→ Running ast-grep rules...").dim());
        }
        
        let runner = AstGrepRunner::new(args.path.clone());
        match runner.run() {
            Ok(report) => {
                combined_report = Some(report);
            }
            Err(e) => {
                eprintln!("{} {}", style("✗").red().bold(), style("ast-grep failed:").red());
                eprintln!("  {}", e);
                eprintln!("  {}", style("Install with: brew install ast-grep").dim());
                if !run_rust {
                    std::process::exit(1);
                }
            }
        }
    }
    
    // Run Rust analyzer for cross-file checks
    if run_rust {
        if args.verbose {
            println!("{}", style("→ Running Rust cross-file analyzer...").dim());
        }
        
        let analyzer = CrossFileAnalyzer::new(args.path.clone());
        let rust_report = analyzer.analyze();
        
        // Merge reports
        combined_report = match combined_report {
            Some(mut existing) => {
                // Merge stats from Rust analyzer (has file counts)
                existing.stats.files_scanned = rust_report.stats.files_scanned;
                existing.stats.types_exported = rust_report.stats.types_exported;
                existing.stats.types_defined = rust_report.stats.types_defined;
                existing.stats.types_in_familiar_core = rust_report.stats.types_in_familiar_core;
                existing.stats.entity_candidates = rust_report.stats.entity_candidates;
                existing.stats.systems_detected = rust_report.stats.systems_detected;
                existing.stats.laws_detected = rust_report.stats.laws_detected;
                existing.stats.decomposition_candidates = rust_report.stats.decomposition_candidates;
                existing.stats.shared_patterns = rust_report.stats.shared_patterns;
                existing.stats.trait_suggestions = rust_report.stats.trait_suggestions;
                
                // Merge issues
                existing.issues.extend(rust_report.issues);
                
                // Deduplicate issues by (file, line, message) - use message for uniqueness
                // since same file:line can have different issues (e.g., multiple MissingGeneration)
                existing.issues.sort_by(|a, b| {
                    (&a.file, a.line, &a.message).cmp(&(&b.file, b.line, &b.message))
                });
                existing.issues.dedup_by(|a, b| {
                    a.file == b.file && a.line == b.line && a.message == b.message
                });
                
                // Update final stats - recalculate from merged issues
                existing.stats.duration_ms = start.elapsed().as_millis() as u64;
                existing.stats.issues_found = existing.issues.len();
                
                // Count from merged issues
                existing.stats.entity_candidates = existing.issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::SuggestEntity { .. }))
                    .count();
                existing.stats.systems_detected = existing.issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::SuggestSystem { .. }))
                    .count();
                existing.stats.laws_detected = existing.issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::SuggestLaw { .. }))
                    .count();
                existing.stats.trait_suggestions = existing.issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::SuggestHasTrait { .. }))
                    .count();
                existing.stats.decomposition_candidates = existing.issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::SuggestDecompose { .. }))
                    .count();
                
                Some(existing)
            }
            None => Some(rust_report),
        };
    }
    
    // Run Database/SeaORM analyzer for service compliance
    if args.database {
        if args.verbose {
            println!("{}", style("→ Running Database/SeaORM compliance analyzer...").dim());
        }
        
        let db_analyzer = DatabaseAnalyzer::new(args.path.clone());
        
        // Get files from walker for analysis
        use familiar_core::analysis::FastWalker;
        let walker = FastWalker::new(args.path.clone());
        let files: Vec<_> = walker.collect_files().into_iter().map(|f| f.path).collect();
        
        let db_issues = db_analyzer.analyze(&files);
        
        // Merge database issues into report
        combined_report = match combined_report {
            Some(mut existing) => {
                existing.issues.extend(db_issues.clone());
                
                // Update database stats
                existing.stats.database_issues = db_issues.len();
                existing.stats.direct_sqlx_usage = db_issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::DirectSqlxUsage { .. }))
                    .count();
                existing.stats.legacy_row_mapping = db_issues.iter()
                    .filter(|i| matches!(i.kind, IssueKind::LegacyRowMapping { .. }))
                    .count();
                existing.stats.issues_found = existing.issues.len();
                
                Some(existing)
            }
            None => {
                // Create new report with just database issues
                let mut stats = familiar_core::analysis::Stats::default();
                stats.database_issues = db_issues.len();
                stats.issues_found = db_issues.len();
                Some(AnalysisReport {
                    issues: db_issues,
                    stats,
                })
            }
        };
    }

    // Run Orphan Schema analyzer
    if args.orphan_schemas {
        if args.verbose {
            println!("{}", style("→ Running orphan schema analyzer...").dim());
        }
        
        let analyzer = CrossFileAnalyzer::new(args.path.clone());
        let (orphan_issues, orphan_stats) = analyzer.analyze_orphan_schemas(
            args.orphan_filter.as_deref()
        );
        
        if !args.quiet {
            println!();
            println!("{}", style("📊 Orphan Schema Analysis").cyan().bold());
            println!("   Schemas with no incoming edges: {}", orphan_stats.total);
            println!();
            println!("   {} {} {}", 
                style("→ Consumer-only:").dim(), 
                orphan_stats.consumer_only, 
                style("(have outgoing refs - normal for tools/contracts)").dim()
            );
            println!("   {} {} {}", 
                style("→ Truly isolated:").cyan(), 
                orphan_stats.truly_isolated, 
                style("(no edges at all - needs attention)").cyan()
            );
            println!();
            println!("   Breakdown of truly isolated:");
            println!("     Expected roots: {} {}", orphan_stats.expected_roots, style("(no action needed)").dim());
            println!("     Connect to graph: {} {}", orphan_stats.connect_graph, style("(used in code)").yellow());
            println!("     Mark deprecated: {} {}", orphan_stats.deprecated, style("(legacy)").dim());
            println!("     Safe to delete: {} {}", orphan_stats.delete, style("(unused)").red());
            println!("   Skipped (primitives/types): {}", orphan_stats.skipped_primitives);
            println!();
        }
        
        // Merge orphan issues into report
        combined_report = match combined_report {
            Some(mut existing) => {
                existing.issues.extend(orphan_issues.clone());
                
                // Update orphan stats
                existing.stats.orphan_schemas = orphan_stats.total;
                existing.stats.orphans_connect_graph = orphan_stats.connect_graph;
                existing.stats.orphans_delete = orphan_stats.delete;
                existing.stats.orphans_deprecated = orphan_stats.deprecated;
                existing.stats.issues_found = existing.issues.len();
                
                Some(existing)
            }
            None => {
                let mut stats = familiar_core::analysis::Stats::default();
                stats.orphan_schemas = orphan_stats.total;
                stats.orphans_connect_graph = orphan_stats.connect_graph;
                stats.orphans_delete = orphan_stats.delete;
                stats.orphans_deprecated = orphan_stats.deprecated;
                stats.issues_found = orphan_issues.len();
                Some(AnalysisReport {
                    issues: orphan_issues,
                    stats,
                })
            }
        };
        
        // Also detect missing JSON schemas (Rust types without schemas)
        let missing_schema_issues = analyzer.detect_missing_json_schemas();
        let missing_count = missing_schema_issues.len();
        
        // And detect isolated types/primitives
        let isolated_issues = analyzer.detect_isolated_schemas(args.orphan_filter.as_deref());
        let isolated_count = isolated_issues.len();
        
        if !args.quiet && (missing_count > 0 || isolated_count > 0) {
            println!("{}", style("📋 Additional Schema Issues").cyan().bold());
            if missing_count > 0 {
                println!("   {} {} {}", 
                    style("→ Missing JSON schemas:").yellow(), 
                    missing_count, 
                    style("(Rust types without schema)").dim()
                );
            }
            if isolated_count > 0 {
                println!("   {} {} {}", 
                    style("→ Isolated types/primitives:").yellow(), 
                    isolated_count, 
                    style("(schemas not referenced)").dim()
                );
            }
            println!();
        }
        
        // Merge into report
        if let Some(ref mut existing) = combined_report {
            existing.issues.extend(missing_schema_issues);
            existing.issues.extend(isolated_issues);
            existing.stats.missing_json_schemas = missing_count;
            existing.stats.isolated_schemas = isolated_count;
            existing.stats.issues_found = existing.issues.len();
        }
        
        // Print detailed breakdown by category if verbose
        if args.verbose && !args.quiet {
            if let Some(ref report) = combined_report {
                let mut by_category: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
                
                for issue in &report.issues {
                    if let IssueKind::OrphanSchema { category, .. } = &issue.kind {
                        by_category.entry(category.clone()).or_default().push(issue);
                    }
                }
                
                println!("{}", style("Breakdown by category:").bold());
                let mut categories: Vec<_> = by_category.iter().collect();
                categories.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
                
                for (category, issues) in categories {
                    let connect = issues.iter()
                        .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::ConnectGraph, .. }))
                        .count();
                    let delete = issues.iter()
                        .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::Delete, .. }))
                        .count();
                    let deprecated = issues.iter()
                        .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::MarkDeprecated, .. }))
                        .count();
                    
                    println!("   {}: {} total ({} connect, {} delete, {} deprecated)",
                        style(category).bold(),
                        issues.len(),
                        connect,
                        delete,
                        deprecated
                    );
                }
                println!();
            }
        }
    }
    
    // Apply fixes if requested
    if let Some(ref report) = combined_report {
        if args.fix || args.fix_unsafe || args.fix_dry_run {
            let dry_run = args.fix_dry_run;
            let fixer = AutoFixer::new(args.path.clone(), dry_run);
            
            if !args.quiet {
                if dry_run {
                    println!("{}", style("🔧 Auto-Fix Preview (dry run)").cyan().bold());
                } else {
                    println!("{}", style("🔧 Applying Auto-Fixes").cyan().bold());
                }
                println!();
            }
            
            // Apply safe fixes
            if args.fix || args.fix_dry_run {
                let safe_summary = fixer.apply_safe_fixes(&report.issues);
                
                if !args.quiet {
                    if safe_summary.safe_applied > 0 || safe_summary.safe_skipped > 0 || dry_run {
                        println!("{}", style("Safe Fixes:").bold());
                        for result in &safe_summary.details {
                            match result {
                                familiar_core::analysis::FixResult::Applied { file, description } => {
                                    println!("  {} {} - {}", 
                                        style("✓").green(), 
                                        file.display(), 
                                        description
                                    );
                                }
                                familiar_core::analysis::FixResult::Skipped { file, reason } => {
                                    if args.verbose {
                                        println!("  {} {} - {}", 
                                            style("⊘").dim(), 
                                            file.display(), 
                                            reason
                                        );
                                    }
                                }
                                familiar_core::analysis::FixResult::Failed { file, error } => {
                                    println!("  {} {} - {}", 
                                        style("✗").red(), 
                                        file.display(), 
                                        error
                                    );
                                }
                            }
                        }
                        println!("  Applied: {}, Skipped: {}, Failed: {}", 
                            safe_summary.safe_applied,
                            safe_summary.safe_skipped,
                            safe_summary.failed
                        );
                        println!();
                    }
                }
            }
            
            // Apply unsafe fixes
            if args.fix_unsafe || (args.fix_dry_run && args.fix_unsafe) {
                let unsafe_summary = fixer.apply_unsafe_fixes(&report.issues, args.confirm);
                
                if !args.quiet {
                    if unsafe_summary.unsafe_applied > 0 || unsafe_summary.unsafe_skipped > 0 {
                        println!("{}", style("Unsafe Fixes:").bold());
                        for result in &unsafe_summary.details {
                            match result {
                                familiar_core::analysis::FixResult::Applied { file, description } => {
                                    println!("  {} {} - {}", 
                                        style("✓").yellow(), 
                                        file.display(), 
                                        description
                                    );
                                }
                                familiar_core::analysis::FixResult::Skipped { file, reason } => {
                                    if args.verbose {
                                        println!("  {} {} - {}", 
                                            style("⊘").dim(), 
                                            file.display(), 
                                            reason
                                        );
                                    }
                                }
                                familiar_core::analysis::FixResult::Failed { file, error } => {
                                    println!("  {} {} - {}", 
                                        style("✗").red(), 
                                        file.display(), 
                                        error
                                    );
                                }
                            }
                        }
                        println!("  Applied: {}, Skipped: {}, Failed: {}", 
                            unsafe_summary.unsafe_applied,
                            unsafe_summary.unsafe_skipped,
                            unsafe_summary.failed
                        );
                        println!();
                    }
                }
            }
        }
    }
    
    // Handle report output
    if let Some(report) = combined_report {
        handle_report(report, &args.path, args.json, args.no_save, args.quiet, args.verbose);
    } else {
        eprintln!("{}", style("No analysis was performed").red());
        std::process::exit(1);
    }
}

fn handle_report(report: AnalysisReport, path: &PathBuf, json_output: bool, no_save: bool, quiet: bool, verbose: bool) {
    let errors = report.issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings = report.issues.iter().filter(|i| i.severity == Severity::Warning).count();
    let infos = report.issues.iter().filter(|i| i.severity == Severity::Info).count();
    
    // Save report to reports/ at project root
    if !no_save {
        let report_dir = path.join("reports");
        
        // Create reports dir if it doesn't exist
        let _ = fs::create_dir_all(&report_dir);
        
        // Save JSON report (source of truth)
        let json_path = report_dir.join("schema_report.json");
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            if fs::write(&json_path, &json).is_ok() {
                if verbose {
                    println!("{} JSON: {}", style("  ✓").green(), json_path.display());
                }
                
                // Generate text report using tabled
                let txt_path = report_dir.join("schema_report.txt");
                let txt_report = format_text_report(&report);
                if fs::write(&txt_path, &txt_report).is_ok() && verbose {
                    println!("{} Text: {}", style("  ✓").green(), txt_path.display());
                }
                
                // Generate HTML report using askama
                let html_path = report_dir.join("schema_report.html");
                let html_template = HtmlReport::from_analysis(&report, &json);
                if let Ok(html_report) = html_template.render() {
                    if fs::write(&html_path, &html_report).is_ok() && verbose {
                        println!("{} HTML: {}", style("  ✓").green(), html_path.display());
                    }
                }
                
                if !verbose && !quiet {
                    println!("{}", style("📁 Reports saved to: reports/").dim());
                }
            }
        }
    }
    
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else if !quiet {
        // Print summary with colors
        println!();
        println!("{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());
        println!("{}", style("📊 Analysis Complete").bold());
        println!("{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());
        println!("   {} {}", style("Files:").dim(), report.stats.files_scanned);
        println!("   {} {}", style("Types:").dim(), report.stats.types_defined);
        println!("   {} {}ms", style("Time:").dim(), report.stats.duration_ms);
        println!();
        
        if errors > 0 {
            println!("   {} {}", style("✗").red().bold(), format!("{} errors", errors).red().bold());
        } else {
            println!("   {} {}", style("✓").green(), "0 errors".green());
        }
        
        if warnings > 0 {
            println!("   {} {}", style("⚠").yellow(), format!("{} warnings", warnings).yellow());
        } else {
            println!("   {} {}", style("✓").green(), "0 warnings".green());
        }
        
        println!("   {} {} suggestions", style("ℹ").blue(), infos);
        println!();
        
        // Show top issues if verbose
        if verbose && !report.issues.is_empty() {
            println!("{}", style("Top issues:").bold());
            for issue in report.issues.iter().take(5) {
                let icon = match issue.severity {
                    Severity::Error => style("✗").red(),
                    Severity::Warning => style("⚠").yellow(),
                    Severity::Info => style("ℹ").blue(),
                };
                println!("  {} {}:{} {}", icon, 
                    style(issue.file.file_name().unwrap_or_default().to_string_lossy()).dim(),
                    issue.line,
                    issue.message);
            }
            if report.issues.len() > 5 {
                println!("  {} more...", style(format!("...and {}", report.issues.len() - 5)).dim());
            }
        }
    }
    
    // Exit with error code if there are errors
    if errors > 0 {
        std::process::exit(1);
    }
}

fn load_config_viewer(args: &ConfigArgs) {
    use std::collections::HashMap;

    println!("🔧 Loading configuration...");

    // Determine service and change to the service directory
    let (service_type, cwd_change) = if let Some(ref file) = args.file {
        println!("📁 Custom config file: {}", file.display());
        ("custom", None)
    } else if let Some(ref service) = args.service {
        match service.as_str() {
            "api" => ("api", Some(PathBuf::from("services/familiar-api"))),
            "worker" => ("worker", Some(PathBuf::from("services/familiar-worker"))),
            "core" => ("core", Some(PathBuf::from("familiar-core"))),
            _ => ("unknown", None),
        }
    } else {
        ("default", None)
    };

    // Change to the service directory if needed
    let original_cwd = if let Some(ref dir) = cwd_change {
        let original = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        std::env::set_current_dir(dir).unwrap_or_else(|_| {});
        Some(original)
    } else {
        None
    };

    // Load config using the config crate with proper environment variable mapping
    use config::{Config as ConfigLoader, Environment, File};

    let mut builder = ConfigLoader::builder()
        .add_source(File::with_name("config").required(false))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .add_source(Environment::with_prefix("").separator("__"));

    // Add support for direct .env variable names (single underscore) based on service type
    match service_type {
        "api" => {
            // API-specific environment variables
            if let Ok(url) = std::env::var("DATABASE_URL") {
                builder = builder.set_override("database.url", url).unwrap();
            }
            if let Ok(url) = std::env::var("WINDMILL_URL") {
                builder = builder.set_override("windmill.url", url).unwrap();
            }
            if let Ok(workspace) = std::env::var("WINDMILL_WORKSPACE") {
                builder = builder.set_override("windmill.workspace", workspace).unwrap();
            }
            if let Ok(token) = std::env::var("WINDMILL_TOKEN") {
                builder = builder.set_override("windmill.token", token).unwrap();
            }
            if let Ok(flow) = std::env::var("WINDMILL_AGENTIC_FLOW") {
                builder = builder.set_override("windmill.agentic_flow", flow).unwrap();
            }
            if let Ok(endpoint) = std::env::var("MINIO_ENDPOINT") {
                builder = builder.set_override("media_store.endpoint", endpoint).unwrap();
            }
            if let Ok(key) = std::env::var("MINIO_ACCESS_KEY") {
                builder = builder.set_override("media_store.access_key", key).unwrap();
            }
            if let Ok(key) = std::env::var("MINIO_SECRET_KEY") {
                builder = builder.set_override("media_store.secret_key", key).unwrap();
            }
            if let Ok(bucket) = std::env::var("MINIO_BUCKET") {
                builder = builder.set_override("media_store.bucket", bucket).unwrap();
            }
            if let Ok(servers) = std::env::var("KAFKA_BOOTSTRAP_SERVERS") {
                builder = builder.set_override("kafka.bootstrap_servers", servers).unwrap();
            }
            if let Ok(group) = std::env::var("KAFKA_GROUP_ID") {
                builder = builder.set_override("kafka.group_id", group).unwrap();
            }
            if let Ok(topic) = std::env::var("KAFKA_COMMANDS_TOPIC") {
                builder = builder.set_override("kafka.commands_topic", topic).unwrap();
            }
            if let Ok(topic) = std::env::var("KAFKA_EVENTS_TOPIC") {
                builder = builder.set_override("kafka.events_topic", topic).unwrap();
            }
            if let Ok(topic) = std::env::var("KAFKA_TRACE_TOPIC") {
                builder = builder.set_override("kafka.trace_topic", topic).unwrap();
            }
            if let Ok(port) = std::env::var("PORT") {
                if let Ok(port_num) = port.parse::<u16>() {
                    builder = builder.set_override("server.port", port_num).unwrap();
                }
            }
        }
        "worker" => {
            // Worker-specific environment variables
            if let Ok(servers) = std::env::var("KAFKA_BOOTSTRAP_SERVERS") {
                builder = builder.set_override("kafka.bootstrap_servers", servers).unwrap();
            }
            if let Ok(group) = std::env::var("KAFKA_GROUP_ID") {
                builder = builder.set_override("kafka.group_id", group).unwrap();
            }
            if let Ok(topic) = std::env::var("KAFKA_COMMANDS_TOPIC") {
                builder = builder.set_override("kafka.commands_topic", topic).unwrap();
            }
            if let Ok(topic) = std::env::var("KAFKA_EVENTS_TOPIC") {
                builder = builder.set_override("kafka.events_topic", topic).unwrap();
            }
            if let Ok(topic) = std::env::var("KAFKA_TRACE_TOPIC") {
                builder = builder.set_override("kafka.trace_topic", topic).unwrap();
            }
            if let Ok(url) = std::env::var("WINDMILL_URL") {
                builder = builder.set_override("windmill.url", url).unwrap();
            }
            if let Ok(workspace) = std::env::var("WINDMILL_WORKSPACE") {
                builder = builder.set_override("windmill.workspace", workspace).unwrap();
            }
            if let Ok(token) = std::env::var("WINDMILL_TOKEN") {
                builder = builder.set_override("windmill.token", token).unwrap();
            }
            if let Ok(url) = std::env::var("DATABASE_URL") {
                builder = builder.set_override("database.url", url).unwrap();
            }
        }
        "core" => {
            // Core-specific environment variables
            if let Ok(url) = std::env::var("WINDMILL_URL") {
                builder = builder.set_override("windmill.url", url).unwrap();
            }
            if let Ok(workspace) = std::env::var("WINDMILL_WORKSPACE") {
                builder = builder.set_override("windmill.workspace", workspace).unwrap();
            }
            if let Ok(token) = std::env::var("WINDMILL_TOKEN") {
                builder = builder.set_override("windmill.token", token).unwrap();
            }
        }
        _ => {}
    }

    let config = match builder.build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    // Schema-first approach: Define sections based on expected config structure
    let sections: Vec<(&str, Vec<&str>)> = vec![
        ("database", vec!["url"]),
        ("windmill", vec!["url", "workspace", "token", "agentic_flow"]),
        ("media_store", vec!["endpoint", "access_key", "secret_key", "bucket", "region"]),
        ("kafka", vec!["bootstrap_servers", "client_id", "group_id", "commands_topic", "events_topic", "trace_topic", "envelope_schema_id", "event_schema_id", "trace_schema_id"]),
        ("server", vec!["port"]),
    ];

    // Convert config to HashMap for easier display
    let config_data: HashMap<String, String> = sections.iter()
        .flat_map(|(section, keys)| {
            keys.iter().map(|key| {
                let full_key = format!("{}.{}", section, key);
                (full_key.clone(), config.get_string(&full_key).unwrap_or_else(|_| "".to_string()))
            }).collect::<Vec<_>>()
        })
        .collect();

    // Schema-first approach: Define sections based on expected config structure
    let sections = vec![
        ("database", vec!["url"]),
        ("windmill", vec!["url", "workspace", "token", "agentic_flow"]),
        ("media_store", vec!["endpoint", "access_key", "secret_key", "bucket", "region"]),
        ("kafka", vec!["bootstrap_servers", "client_id", "group_id", "commands_topic", "events_topic", "trace_topic", "envelope_schema_id", "event_schema_id", "trace_schema_id"]),
        ("server", vec!["port"]),
    ];

    let mut found_values = HashMap::new();

    for (section_name, keys) in sections {
        println!("\n📋 {}:", section_name.to_uppercase());
        let mut section_has_values = false;

        for key in keys {
            let full_key = format!("{}.{}", section_name, key);
            if let Some(value) = config_data.get(&full_key) {
                section_has_values = true;
                let display_value = if key.contains("token") || key.contains("secret") || key.contains("key") {
                    if args.show_secrets {
                        format!("🔓 {}", value)
                    } else {
                        "🔒 [hidden - use --show-secrets]".to_string()
                    }
                } else {
                    value.clone()
                };
                found_values.insert(full_key, display_value.clone());
                println!("  {}: {}", key, display_value);
            }
        }

        if !section_has_values {
            println!("  (no values configured)");
        }
    }

    // Warn about secrets
    if args.show_secrets {
        eprintln!("\n⚠️  WARNING: Secrets are visible above!");
    } else {
        let secret_keys: Vec<&String> = found_values.keys()
            .filter(|k: &&String| k.contains("token") || k.contains("secret") || k.contains("key"))
            .collect();
        if !secret_keys.is_empty() {
            println!("\n🔒 {} secret values hidden. Use --show-secrets to reveal.", secret_keys.len());
        }
    }

    println!("\n✅ Configuration loaded successfully");

    // Restore original working directory
    if let Some(original) = original_cwd {
        std::env::set_current_dir(original).unwrap_or_else(|_| {});
    }
}
