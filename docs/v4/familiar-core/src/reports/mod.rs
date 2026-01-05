//! Report Generation Module
//!
//! Uses askama for HTML templates and tabled for text tables.
//! This module reduces ~800 lines of inline formatting to ~150 lines.

use askama::Template;
use crate::analysis::{AnalysisReport, Issue, IssueKind, OrphanRecommendation, Severity, Stats};
use tabled::{Table, Tabled, settings::Style};

// ============================================================================
// HTML Report (askama)
// ============================================================================

/// A stat card for the dashboard
#[derive(Clone)]
pub struct StatCard {
    pub value: String,
    pub label: &'static str,
    pub class: &'static str,
}

/// An issue formatted for display
#[derive(Clone)]
pub struct DisplayIssue {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: Option<String>,
}

/// A group of issues by category
#[derive(Clone)]
pub struct IssueGroup {
    pub icon: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: Option<&'static str>,
    pub issues: Vec<DisplayIssue>,
}

/// Data for a single tab (Core or Services)
#[derive(Clone)]
pub struct TabData {
    pub name: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub errors: Vec<DisplayIssue>,
    pub warnings: Vec<DisplayIssue>,
    pub infos: Vec<DisplayIssue>,
    pub error_groups: Vec<IssueGroup>,
    pub warning_groups: Vec<IssueGroup>,
    pub info_groups: Vec<IssueGroup>,
}

/// HTML report template using askama
#[derive(Template)]
#[template(path = "report.html")]
pub struct HtmlReport {
    pub timestamp: String,
    pub stats_cards: Vec<StatCard>,
    pub core_tab: TabData,
    pub services_tab: TabData,
    pub escaped_json: String,
}

/// Check if a file path is in familiar-core (the source of truth library)
fn is_core_file(path: &str) -> bool {
    path.contains("/familiar-core/") || 
    path.contains("/familiar-contracts/") || 
    path.contains("/familiar-primitives/")
}

impl HtmlReport {
    /// Create HTML report from analysis report
    pub fn from_analysis(report: &AnalysisReport, json_content: &str) -> Self {
        // Split issues by core vs services
        let (core_issues, services_issues): (Vec<_>, Vec<_>) = report.issues.iter()
            .partition(|i| is_core_file(&i.file.display().to_string()));
        
        let core_tab = build_tab_data(
            "Core",
            "📦",
            "familiar-core, familiar-contracts, familiar-primitives - Source of truth",
            &core_issues,
        );
        
        let services_tab = build_tab_data(
            "Services", 
            "🚀",
            "familiar-api, familiar-worker, windmill - Consumers of core",
            &services_issues,
        );

        let escaped_json = json_content
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace("${", "\\${");

        // Overall stats
        let total_errors = core_tab.error_count + services_tab.error_count;
        let total_warnings = core_tab.warning_count + services_tab.warning_count;
        let total_infos = core_tab.info_count + services_tab.info_count;

        Self {
            timestamp: chrono_lite_now(),
            stats_cards: build_stats_cards(&report.stats, total_errors, total_warnings, total_infos),
            core_tab,
            services_tab,
            escaped_json,
        }
    }
}

fn build_tab_data(
    name: &'static str,
    icon: &'static str,
    description: &'static str,
    issues: &[&Issue],
) -> TabData {
    let errors: Vec<_> = issues.iter()
        .filter(|i| i.severity == Severity::Error)
        .copied()
        .collect();
    let warnings: Vec<_> = issues.iter()
        .filter(|i| i.severity == Severity::Warning)
        .copied()
        .collect();
    let infos: Vec<_> = issues.iter()
        .filter(|i| i.severity == Severity::Info)
        .copied()
        .collect();

    TabData {
        name,
        icon,
        description,
        error_count: errors.len(),
        warning_count: warnings.len(),
        info_count: infos.len(),
        errors: errors.iter().map(|i| to_display_issue(i)).collect(),
        warnings: warnings.iter().map(|i| to_display_issue(i)).collect(),
        infos: infos.iter().map(|i| to_display_issue(i)).collect(),
        error_groups: build_error_groups(&errors),
        warning_groups: build_warning_groups(&warnings),
        info_groups: build_info_groups(&infos),
    }
}

fn to_display_issue(issue: &Issue) -> DisplayIssue {
    DisplayIssue {
        file: issue.file.display().to_string(),
        line: issue.line,
        message: issue.message.clone(),
        fix: issue.fix.as_ref().map(|f| f.description.clone()),
    }
}

fn build_stats_cards(stats: &Stats, errors: usize, warnings: usize, infos: usize) -> Vec<StatCard> {
    let mut cards = vec![
        // Primary metrics - always shown
        StatCard { 
            value: errors.to_string(), 
            label: "Errors", 
            class: if errors == 0 { " success" } else { " error" } 
        },
        StatCard { 
            value: warnings.to_string(), 
            label: "Warnings", 
            class: if warnings == 0 { " success" } else { " warning" } 
        },
        StatCard { 
            value: infos.to_string(), 
            label: "Info", 
            class: "" 
        },
    ];
    
    // Secondary metrics - only shown if non-zero or relevant
    if stats.communication_violations > 0 {
        cards.push(StatCard { 
            value: stats.communication_violations.to_string(), 
            label: "Comm Violations", 
            class: " error" 
        });
    }
    
    if stats.database_issues > 0 {
        cards.push(StatCard { 
            value: stats.database_issues.to_string(), 
            label: "Database Issues", 
            class: " warning" 
        });
    }
    
    if stats.kafka_issues > 0 {
        cards.push(StatCard { 
            value: stats.kafka_issues.to_string(), 
            label: "Kafka Issues", 
            class: " warning" 
        });
    }
    
    if stats.entity_candidates > 0 {
        cards.push(StatCard { 
            value: stats.entity_candidates.to_string(), 
            label: "Entities", 
            class: "" 
        });
    }
    
    // Orphan schema stats
    if stats.orphan_schemas > 0 {
        cards.push(StatCard { 
            value: stats.orphan_schemas.to_string(), 
            label: "Orphan Schemas", 
            class: " warning" 
        });
    }
    
    if stats.orphans_connect_graph > 0 {
        cards.push(StatCard { 
            value: stats.orphans_connect_graph.to_string(), 
            label: "Need Graph Link", 
            class: "" 
        });
    }
    
    if stats.orphans_delete > 0 {
        cards.push(StatCard { 
            value: stats.orphans_delete.to_string(), 
            label: "Safe to Delete", 
            class: " warning" 
        });
    }
    
    if stats.missing_json_schemas > 0 {
        cards.push(StatCard { 
            value: stats.missing_json_schemas.to_string(), 
            label: "Missing Schemas", 
            class: " warning" 
        });
    }
    
    if stats.isolated_schemas > 0 {
        cards.push(StatCard { 
            value: stats.isolated_schemas.to_string(), 
            label: "Isolated Types", 
            class: " warning" 
        });
    }
    
    // Duration - always shown
    cards.push(StatCard { 
        value: format!("{}ms", stats.duration_ms), 
        label: "Duration", 
        class: "" 
    });
    
    cards
}

fn build_error_groups(errors: &[&Issue]) -> Vec<IssueGroup> {
    vec![
        IssueGroup {
            icon: "🔌",
            title: "Communication Violations",
            category: "communication",
            description: Some("Workers must use Kafka for internal communication - HTTP is not allowed."),
            issues: errors.iter()
                .filter(|i| matches!(i.kind, IssueKind::DirectHttpInWorker { .. } | IssueKind::HttpClientInWorkerStruct { .. } | IssueKind::InterServiceBypassKafka { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🗃️",
            title: "Database/SeaORM Violations",
            category: "database",
            description: Some("Services must use TigerDataStore from familiar-core, not direct sqlx."),
            issues: errors.iter()
                .filter(|i| matches!(i.kind, IssueKind::DirectSqlxUsage { .. } | IssueKind::DirectPoolAccess { .. } | IssueKind::EntityInService { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📨",
            title: "Kafka/Protobuf Errors",
            category: "kafka",
            description: None,
            issues: errors.iter()
                .filter(|i| matches!(i.kind, IssueKind::MissingProtobufSchema { .. } | IssueKind::MissingKafkaKey { .. } | IssueKind::CodegenPathViolation { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔄",
            title: "Generation Sync",
            category: "generation",
            description: None,
            issues: errors.iter()
                .filter(|i| matches!(i.kind, IssueKind::MissingGeneration { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔗",
            title: "Broken Imports",
            category: "import",
            description: None,
            issues: errors.iter()
                .filter(|i| matches!(i.kind, IssueKind::BrokenImport { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📋",
            title: "Missing Schema",
            category: "schema",
            description: None,
            issues: errors.iter()
                .filter(|i| matches!(i.kind, IssueKind::MissingSchema { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "❓",
            title: "Other",
            category: "other",
            description: None,
            issues: errors.iter()
                .filter(|i| !matches!(i.kind, 
                    IssueKind::MissingGeneration { .. } | IssueKind::BrokenImport { .. } | IssueKind::MissingSchema { .. } |
                    IssueKind::MissingProtobufSchema { .. } | IssueKind::MissingKafkaKey { .. } | IssueKind::CodegenPathViolation { .. } |
                    IssueKind::DirectHttpInWorker { .. } | IssueKind::HttpClientInWorkerStruct { .. } | IssueKind::InterServiceBypassKafka { .. } |
                    IssueKind::DirectSqlxUsage { .. } | IssueKind::DirectPoolAccess { .. } | IssueKind::EntityInService { .. }
                ))
                .map(|i| to_display_issue(i))
                .collect(),
        },
    ]
}

fn build_warning_groups(warnings: &[&Issue]) -> Vec<IssueGroup> {
    vec![
        IssueGroup {
            icon: "📝",
            title: "Missing JSON Schemas",
            category: "missing-schema",
            description: Some("Rust types without corresponding JSON schema - need to be exported"),
            issues: warnings.iter()
                .filter(|i| matches!(&i.kind, IssueKind::MissingJsonSchema { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔌",
            title: "Isolated Types/Primitives",
            category: "isolated-schema",
            description: Some("Schema types not referenced by any other schema - need $ref connections"),
            issues: warnings.iter()
                .filter(|i| matches!(&i.kind, IssueKind::IsolatedSchema { used_in_code: true, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔗",
            title: "Orphan Schemas (Delete)",
            category: "orphan-delete",
            description: Some("Schemas not used in code or graph - safe to delete"),
            issues: warnings.iter()
                .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::Delete, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔌",
            title: "Communication Warnings",
            category: "communication",
            description: None,
            issues: warnings.iter()
                .filter(|i| matches!(i.kind, IssueKind::DirectHttpInWorker { .. } | IssueKind::HttpClientInWorkerStruct { .. } | IssueKind::InterServiceBypassKafka { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🗃️",
            title: "Database Warnings",
            category: "database",
            description: None,
            issues: warnings.iter()
                .filter(|i| matches!(i.kind, IssueKind::LegacyRowMapping { .. } | IssueKind::BypassingStore { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📨",
            title: "Kafka Warnings",
            category: "kafka",
            description: None,
            issues: warnings.iter()
                .filter(|i| matches!(i.kind, IssueKind::ManualKafkaImplementation { .. } | IssueKind::JsonSerializationInsteadOfProtobuf { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔢",
            title: "Raw Primitives",
            category: "primitive",
            description: None,
            issues: warnings.iter()
                .filter(|i| matches!(i.kind, IssueKind::RawPrimitive { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📄",
            title: "Duplicate Types",
            category: "schema",
            description: None,
            issues: warnings.iter()
                .filter(|i| matches!(i.kind, IssueKind::DuplicateType { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📖",
            title: "OpenAPI Gaps",
            category: "schema",
            description: None,
            issues: warnings.iter()
                .filter(|i| matches!(i.kind, IssueKind::MissingOpenApiDerive { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "❓",
            title: "Other",
            category: "other",
            description: None,
            issues: warnings.iter()
                .filter(|i| !matches!(i.kind, 
                    IssueKind::RawPrimitive { .. } | IssueKind::DuplicateType { .. } | IssueKind::MissingOpenApiDerive { .. } |
                    IssueKind::ManualKafkaImplementation { .. } | IssueKind::JsonSerializationInsteadOfProtobuf { .. } |
                    IssueKind::DirectHttpInWorker { .. } | IssueKind::HttpClientInWorkerStruct { .. } | IssueKind::InterServiceBypassKafka { .. } |
                    IssueKind::LegacyRowMapping { .. } | IssueKind::BypassingStore { .. } |
                    IssueKind::OrphanSchema { .. }
                ))
                .map(|i| to_display_issue(i))
                .collect(),
        },
    ]
}

fn build_info_groups(infos: &[&Issue]) -> Vec<IssueGroup> {
    vec![
        IssueGroup {
            icon: "📭",
            title: "Isolated Schemas (Unused)",
            category: "isolated-unused",
            description: Some("Schema types not referenced by schemas or code - may be candidates for deletion"),
            issues: infos.iter()
                .filter(|i| matches!(&i.kind, IssueKind::IsolatedSchema { used_in_code: false, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🔗",
            title: "Orphan Schemas (Connect Graph)",
            category: "orphan-connect",
            description: Some("Schemas used in code but not connected in the schema graph - run --fix to add x-familiar-* extensions"),
            issues: infos.iter()
                .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::ConnectGraph, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📦",
            title: "Orphan Schemas (Deprecated)",
            category: "orphan-deprecated",
            description: Some("Legacy schemas from deprecated systems (e.g., Windmill) - run --fix to mark as deprecated"),
            issues: infos.iter()
                .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::MarkDeprecated, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "✅",
            title: "Orphan Schemas (Expected Roots)",
            category: "orphan-root",
            description: Some("Infrastructure schemas that are expected to be roots (ecs/, queues/, nodes/, systems/)"),
            issues: infos.iter()
                .filter(|i| matches!(&i.kind, IssueKind::OrphanSchema { recommendation: OrphanRecommendation::ExpectedRoot, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "⚙️",
            title: "ECS Systems",
            category: "system",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::SuggestSystem { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "⚖️",
            title: "Laws",
            category: "law",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::SuggestLaw { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "🗄️",
            title: "Entity Candidates",
            category: "entity",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::SuggestEntity { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "💡",
            title: "Schema Suggestions",
            category: "schema",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::SuggestSchema { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📭",
            title: "Unused Schemas",
            category: "schema",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::UnusedSchema { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📦",
            title: "Centralize Types",
            category: "centralize",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::SuggestCentralize { .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📋",
            title: "Local Types (Schema Exists)",
            category: "schema-exists",
            description: Some("These types exist in familiar-schemas - use the schema instead of local definition"),
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::LocalTypeNotInSchema { schema_exists: true, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "📝",
            title: "Local Types (Add to Schema)",
            category: "schema-needed",
            description: Some("These types should be added to familiar-schemas for cross-language consistency"),
            issues: infos.iter()
                .filter(|i| matches!(i.kind, IssueKind::LocalTypeNotInSchema { schema_exists: false, .. }))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        #[allow(deprecated)]
        IssueGroup {
            icon: "🐍",
            title: "Python Codegen (Legacy)",
            category: "python",
            description: None,
            issues: infos.iter()
                .filter(|i| matches!(&i.kind, IssueKind::LocalTypeNotInSchema { language, .. } if language == "python"))
                .map(|i| to_display_issue(i))
                .collect(),
        },
        IssueGroup {
            icon: "❓",
            title: "Other",
            category: "other",
            description: None,
            issues: infos.iter()
                .filter(|i| !matches!(i.kind, 
                    IssueKind::SuggestSystem { .. } | IssueKind::SuggestLaw { .. } | 
                    IssueKind::SuggestEntity { .. } | IssueKind::SuggestSchema { .. } | 
                    IssueKind::UnusedSchema { .. } | IssueKind::SuggestCentralize { .. } |
                    IssueKind::LocalTypeNotInSchema { .. } |
                    IssueKind::OrphanSchema { .. }
                ))
                .map(|i| to_display_issue(i))
                .collect(),
        },
    ]
}

// ============================================================================
// Text Report (tabled)
// ============================================================================

/// Row for the stats table
#[derive(Tabled)]
pub struct StatsRow {
    #[tabled(rename = "Metric")]
    pub metric: &'static str,
    #[tabled(rename = "Value")]
    pub value: String,
}

/// Row for an issue table
#[derive(Tabled)]
pub struct IssueRow {
    #[tabled(rename = "File")]
    pub file: String,
    #[tabled(rename = "Line")]
    pub line: String,
    #[tabled(rename = "Message")]
    pub message: String,
}

impl IssueRow {
    pub fn from_issue(issue: &Issue) -> Self {
        Self {
            file: truncate_path(&issue.file.display().to_string(), 50),
            line: issue.line.to_string(),
            message: truncate_str(&issue.message, 60),
        }
    }
}

/// Generate a formatted text report using tabled
pub fn format_text_report(report: &AnalysisReport) -> String {
    let mut out = String::new();
    
    out.push_str("Schema Analysis Report\n");
    out.push_str("======================\n\n");
    out.push_str(&format!("Generated: {}\n\n", chrono_lite_now()));
    
    // Stats table - only show non-zero values
    let errors = report.issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings_count = report.issues.iter().filter(|i| i.severity == Severity::Warning).count();
    let infos_count = report.issues.iter().filter(|i| i.severity == Severity::Info).count();
    
    let mut stats_rows = vec![
        StatsRow { metric: "Errors", value: errors.to_string() },
        StatsRow { metric: "Warnings", value: warnings_count.to_string() },
        StatsRow { metric: "Info", value: infos_count.to_string() },
    ];
    
    if report.stats.communication_violations > 0 {
        stats_rows.push(StatsRow { metric: "Comm Violations", value: report.stats.communication_violations.to_string() });
    }
    if report.stats.database_issues > 0 {
        stats_rows.push(StatsRow { metric: "Database Issues", value: report.stats.database_issues.to_string() });
    }
    if report.stats.kafka_issues > 0 {
        stats_rows.push(StatsRow { metric: "Kafka Issues", value: report.stats.kafka_issues.to_string() });
    }
    if report.stats.entity_candidates > 0 {
        stats_rows.push(StatsRow { metric: "Entities", value: report.stats.entity_candidates.to_string() });
    }
    
    stats_rows.push(StatsRow { metric: "Duration", value: format!("{}ms", report.stats.duration_ms) });
    
    let stats_table = Table::new(stats_rows)
        .with(Style::rounded())
        .to_string();
    out.push_str(&stats_table);
    out.push_str("\n\n");
    
    // Group issues by severity
    let errors: Vec<_> = report.issues.iter().filter(|i| i.severity == Severity::Error).collect();
    let warnings: Vec<_> = report.issues.iter().filter(|i| i.severity == Severity::Warning).collect();
    let infos: Vec<_> = report.issues.iter().filter(|i| i.severity == Severity::Info).collect();
    
    // Errors
    if !errors.is_empty() {
        out.push_str(&format!("\n🔴 ERRORS ({})\n", errors.len()));
        out.push_str(&"─".repeat(60));
        out.push_str("\n");
        
        // Communication errors
        let comm_errors: Vec<_> = errors.iter()
            .filter(|i| matches!(i.kind, IssueKind::DirectHttpInWorker { .. } | IssueKind::HttpClientInWorkerStruct { .. } | IssueKind::InterServiceBypassKafka { .. }))
            .collect();
        if !comm_errors.is_empty() {
            out.push_str(&format!("\n🔌 Communication Violations ({})\n", comm_errors.len()));
            let rows: Vec<_> = comm_errors.iter().map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            out.push_str("\n");
        }
        
        // Other errors
        let other_errors: Vec<_> = errors.iter()
            .filter(|i| !matches!(i.kind, IssueKind::DirectHttpInWorker { .. } | IssueKind::HttpClientInWorkerStruct { .. } | IssueKind::InterServiceBypassKafka { .. }))
            .collect();
        if !other_errors.is_empty() {
            out.push_str(&format!("\n❓ Other Errors ({})\n", other_errors.len()));
            let rows: Vec<_> = other_errors.iter().map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            out.push_str("\n");
        }
    }
    
    // Warnings
    if !warnings.is_empty() {
        out.push_str(&format!("\n🟡 WARNINGS ({})\n", warnings.len()));
        out.push_str(&"─".repeat(60));
        out.push_str("\n");
        
        // Group by type and show tables
        let openapi: Vec<_> = warnings.iter()
            .filter(|i| matches!(i.kind, IssueKind::MissingOpenApiDerive { .. }))
            .collect();
        if !openapi.is_empty() {
            out.push_str(&format!("\n📖 OpenAPI Gaps ({})\n", openapi.len()));
            let rows: Vec<_> = openapi.iter().take(20).map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            if openapi.len() > 20 {
                out.push_str(&format!("\n... and {} more\n", openapi.len() - 20));
            }
            out.push_str("\n");
        }
        
        let primitives: Vec<_> = warnings.iter()
            .filter(|i| matches!(i.kind, IssueKind::RawPrimitive { .. }))
            .collect();
        if !primitives.is_empty() {
            out.push_str(&format!("\n🔢 Raw Primitives ({})\n", primitives.len()));
            let rows: Vec<_> = primitives.iter().take(20).map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            if primitives.len() > 20 {
                out.push_str(&format!("\n... and {} more\n", primitives.len() - 20));
            }
            out.push_str("\n");
        }
    }
    
    // Info
    if !infos.is_empty() {
        out.push_str(&format!("\n🔵 INFO & SUGGESTIONS ({})\n", infos.len()));
        out.push_str(&"─".repeat(60));
        out.push_str("\n");
        
        let systems: Vec<_> = infos.iter()
            .filter(|i| matches!(i.kind, IssueKind::SuggestSystem { .. }))
            .collect();
        if !systems.is_empty() {
            out.push_str(&format!("\n⚙️ ECS Systems ({})\n", systems.len()));
            let rows: Vec<_> = systems.iter().map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            out.push_str("\n");
        }
        
        let laws: Vec<_> = infos.iter()
            .filter(|i| matches!(i.kind, IssueKind::SuggestLaw { .. }))
            .collect();
        if !laws.is_empty() {
            out.push_str(&format!("\n⚖️ Laws ({})\n", laws.len()));
            let rows: Vec<_> = laws.iter().map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            out.push_str("\n");
        }
        
        let entities: Vec<_> = infos.iter()
            .filter(|i| matches!(i.kind, IssueKind::SuggestEntity { .. }))
            .collect();
        if !entities.is_empty() {
            out.push_str(&format!("\n🗄️ Entity Candidates ({})\n", entities.len()));
            let rows: Vec<_> = entities.iter().take(20).map(|i| IssueRow::from_issue(i)).collect();
            out.push_str(&Table::new(rows).with(Style::rounded()).to_string());
            if entities.len() > 20 {
                out.push_str(&format!("\n... and {} more\n", entities.len() - 20));
            }
            out.push_str("\n");
        }
    }
    
    out
}

// ============================================================================
// Helpers
// ============================================================================

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn truncate_path(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Keep the end of the path (most relevant part)
        format!("...{}", &s[s.len() - max_len + 3..])
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let mins = (rem % 3600) / 60;
    let secs_of_day = rem % 60;
    
    let mut year = 1970i64;
    let mut remaining_days = days as i64;
    
    loop {
        let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
        if remaining_days < days_in_year { break; }
        remaining_days -= days_in_year;
        year += 1;
    }
    
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_months: [i64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut month = 1;
    for dim in days_in_months {
        if remaining_days < dim { break; }
        remaining_days -= dim;
        month += 1;
    }
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", 
        year, month, remaining_days + 1, hours, mins, secs_of_day)
}

// Custom filter for askama to truncate issue lists
mod filters {
    use super::DisplayIssue;
    
    pub fn truncate_issues(issues: &[DisplayIssue]) -> askama::Result<Vec<&DisplayIssue>> {
        Ok(issues.iter().take(50).collect())
    }
}

