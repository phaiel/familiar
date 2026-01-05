//! Minerva CLI Structure
//!
//! Defines the command-line interface for the familiar-worker binary.
//! CLI-only mode: Windmill (the Orchestrator) invokes Minerva commands.
//!
//! ## The Evaluator Pattern
//!
//! Each domain has `evaluate-*` and `execute-*` subcommands:
//! - `evaluate-*`: Returns EvaluationResult with next_step (LOOM, DIRECT, etc.)
//! - `execute-*`: Performs the actual work
//!
//! ## Usage
//!
//! ```bash
//! # Evaluate a signup (returns EvaluationResult)
//! minerva onboarding evaluate-signup --input '{"email": "user@example.com", ...}'
//!
//! # Execute a signup (performs the work)
//! minerva onboarding execute-signup --input '{"email": "user@example.com", ...}'
//!
//! # Fates pipeline
//! minerva fates pipeline --input '{"content": "...", ...}'
//!
//! # Health check
//! minerva maintenance health
//! ```

use clap::{Parser, Subcommand};

/// Minerva - The Master Weaver
///
/// CLI-only executor for Windmill (the Orchestrator).
/// Each command reads JSON input and outputs JSON to stdout.
#[derive(Parser, Debug)]
#[command(
    name = "minerva",
    version,
    about = "Minerva - The Master Weaver",
    long_about = "Minerva executes the strategic will of the Orchestrator (Windmill).\n\n\
        Each command reads --input JSON and writes JSON to stdout.\n\
        Exit code 0 = success, non-zero = failure."
)]
pub struct Cli {
    /// JSON input for the command
    #[arg(long, env = "MINERVA_INPUT")]
    pub input: Option<String>,

    /// Path to JSON input file (alternative to --input)
    #[arg(long)]
    pub input_file: Option<std::path::PathBuf>,

    /// Enable verbose logging
    #[arg(short, long, env = "MINERVA_VERBOSE")]
    pub verbose: bool,

    #[command(subcommand)]
    pub domain: Domain,
}

/// Domain subcommands - each represents a bounded context
#[derive(Subcommand, Debug, Clone)]
pub enum Domain {
    /// AI Pipeline (Gate, Morta, Decima, Nona)
    /// The Fates process messages through the agentic pipeline
    Fates {
        #[command(subcommand)]
        action: FatesAction,
    },

    /// Identity & Provisioning
    /// User signup, family creation, invitation acceptance
    Onboarding {
        #[command(subcommand)]
        action: OnboardingAction,
    },

    /// VAE & Manifold Physics
    /// Position calculations, binding operations, field excitations
    Manifold {
        #[command(subcommand)]
        action: ManifoldAction,
    },

    /// System Maintenance
    /// Data cleanup, health checks
    Maintenance {
        #[command(subcommand)]
        action: MaintenanceAction,
    },
}

// =============================================================================
// Fates Domain Actions
// =============================================================================

/// Actions within the Fates (AI Pipeline) domain
#[derive(Subcommand, Debug, Clone)]
pub enum FatesAction {
    /// Gate: Input classification and routing
    Gate,
    /// Morta: Content segmentation
    Morta,
    /// Decima: Entity extraction
    Decima,
    /// Nona: Response generation
    Nona,
    /// Run the full Fates pipeline
    Pipeline,
    /// Evaluate message for routing (Evaluator Pattern)
    #[command(name = "evaluate")]
    Evaluate,
}

// =============================================================================
// Onboarding Domain Actions
// =============================================================================

/// Actions within the Onboarding domain
/// 
/// Includes both Evaluator Pattern commands (evaluate-*, execute-*) and
/// direct database operation commands for Windmill script replacement.
#[derive(Subcommand, Debug, Clone)]
pub enum OnboardingAction {
    // =========================================================================
    // Evaluator Pattern Commands (high-level workflow steps)
    // =========================================================================
    
    /// Evaluate signup request (Evaluator Pattern)
    #[command(name = "evaluate-signup")]
    EvaluateSignup,
    /// Execute a new user signup
    #[command(name = "execute-signup")]
    ExecuteSignup,
    /// Evaluate family creation (Evaluator Pattern)
    #[command(name = "evaluate-create-family")]
    EvaluateCreateFamily,
    /// Execute family/tenant creation
    #[command(name = "execute-create-family")]
    ExecuteCreateFamily,
    /// Evaluate invitation acceptance (Evaluator Pattern)
    #[command(name = "evaluate-accept-invitation")]
    EvaluateAcceptInvitation,
    /// Execute invitation acceptance
    #[command(name = "execute-accept-invitation")]
    ExecuteAcceptInvitation,
    /// Process any pending onboarding task
    Process,

    // =========================================================================
    // Direct Database Operation Commands (Windmill script replacements)
    // =========================================================================
    
    /// Check if email already exists in the system
    #[command(name = "check-email")]
    CheckEmail {
        #[arg(long)]
        email: String,
    },
    
    /// Create a new user record
    #[command(name = "create-user")]
    CreateUser {
        #[arg(long)]
        email: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        password_hash: Option<String>,
    },
    
    /// Create an auth session for a user
    #[command(name = "create-session")]
    CreateSession {
        #[arg(long)]
        user_id: String,
    },
    
    /// Record GDPR consent records for a user
    #[command(name = "record-consent")]
    RecordConsent {
        #[arg(long)]
        user_id: String,
        /// JSON array of consent records
        #[arg(long)]
        consents_json: String,
    },
    
    /// Validate an invitation code (check if it exists and is valid)
    #[command(name = "validate-invite")]
    ValidateInvite {
        #[arg(long)]
        code: String,
    },
    
    /// Validate invitation for a specific user
    #[command(name = "validate-invitation")]
    ValidateInvitation {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        invite_code: String,
    },
    
    /// Check if a user needs a family (has no tenant membership)
    #[command(name = "check-needs-family")]
    CheckNeedsFamily {
        #[arg(long)]
        user_id: String,
    },
    
    /// Create a new tenant (family)
    #[command(name = "create-tenant")]
    CreateTenant {
        #[arg(long)]
        name: String,
    },
    
    /// Add a user as a member to a tenant
    #[command(name = "add-member")]
    AddMember {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        user_id: String,
        #[arg(long, value_parser = ["admin", "member", "guest"])]
        role: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        email: String,
    },
    
    /// Set a user's primary tenant
    #[command(name = "set-primary-tenant")]
    SetPrimaryTenant {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        tenant_id: String,
    },
    
    /// Create a channel in a tenant
    #[command(name = "create-channel")]
    CreateChannel {
        #[arg(long)]
        tenant_id: String,
        #[arg(long, value_parser = ["personal", "family", "shared"])]
        channel_type: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        owner_id: Option<String>,
    },
    
    /// Increment invitation usage count
    #[command(name = "increment-invite")]
    IncrementInvite {
        #[arg(long)]
        invitation_id: String,
    },
    
    /// Create an audit log entry
    #[command(name = "audit-log")]
    AuditLog {
        #[arg(long)]
        action: String,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long)]
        user_email: Option<String>,
        #[arg(long)]
        resource_type: Option<String>,
        #[arg(long)]
        resource_id: Option<String>,
        #[arg(long)]
        metadata_json: Option<String>,
    },
}

// =============================================================================
// Manifold Domain Actions
// =============================================================================

/// Actions within the Manifold (Physics) domain
#[derive(Subcommand, Debug, Clone)]
pub enum ManifoldAction {
    /// Calculate VAE positions
    Position,
    /// Perform cognitive binding
    Bind,
    /// Calculate field excitations
    Excite,
    /// Run full physics step
    Step,
}

// =============================================================================
// Maintenance Domain Actions
// =============================================================================

/// Actions within the Maintenance domain
#[derive(Subcommand, Debug, Clone)]
pub enum MaintenanceAction {
    /// Clean up old data
    Cleanup,
    /// Health check
    Health,
    /// Export metrics
    Metrics,
}

// =============================================================================
// Helper implementations
// =============================================================================

impl Cli {
    /// Get the domain name as a string for logging
    pub fn domain_name(&self) -> &'static str {
        match &self.domain {
            Domain::Fates { .. } => "fates",
            Domain::Onboarding { .. } => "onboarding",
            Domain::Manifold { .. } => "manifold",
            Domain::Maintenance { .. } => "maintenance",
        }
    }

    /// Get the action name as a string for logging
    pub fn action_name(&self) -> &'static str {
        match &self.domain {
            Domain::Fates { action } => match action {
                FatesAction::Gate => "gate",
                FatesAction::Morta => "morta",
                FatesAction::Decima => "decima",
                FatesAction::Nona => "nona",
                FatesAction::Pipeline => "pipeline",
                FatesAction::Evaluate => "evaluate",
            },
            Domain::Onboarding { action } => match action {
                // Evaluator Pattern commands
                OnboardingAction::EvaluateSignup => "evaluate-signup",
                OnboardingAction::ExecuteSignup => "execute-signup",
                OnboardingAction::EvaluateCreateFamily => "evaluate-create-family",
                OnboardingAction::ExecuteCreateFamily => "execute-create-family",
                OnboardingAction::EvaluateAcceptInvitation => "evaluate-accept-invitation",
                OnboardingAction::ExecuteAcceptInvitation => "execute-accept-invitation",
                OnboardingAction::Process => "process",
                // Direct database operation commands
                OnboardingAction::CheckEmail { .. } => "check-email",
                OnboardingAction::CreateUser { .. } => "create-user",
                OnboardingAction::CreateSession { .. } => "create-session",
                OnboardingAction::RecordConsent { .. } => "record-consent",
                OnboardingAction::ValidateInvite { .. } => "validate-invite",
                OnboardingAction::ValidateInvitation { .. } => "validate-invitation",
                OnboardingAction::CheckNeedsFamily { .. } => "check-needs-family",
                OnboardingAction::CreateTenant { .. } => "create-tenant",
                OnboardingAction::AddMember { .. } => "add-member",
                OnboardingAction::SetPrimaryTenant { .. } => "set-primary-tenant",
                OnboardingAction::CreateChannel { .. } => "create-channel",
                OnboardingAction::IncrementInvite { .. } => "increment-invite",
                OnboardingAction::AuditLog { .. } => "audit-log",
            },
            Domain::Manifold { action } => match action {
                ManifoldAction::Position => "position",
                ManifoldAction::Bind => "bind",
                ManifoldAction::Excite => "excite",
                ManifoldAction::Step => "step",
            },
            Domain::Maintenance { action } => match action {
                MaintenanceAction::Cleanup => "cleanup",
                MaintenanceAction::Health => "health",
                MaintenanceAction::Metrics => "metrics",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
