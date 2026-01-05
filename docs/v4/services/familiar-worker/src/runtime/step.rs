//! Step Mode Runtime (CLI-Only)
//!
//! One-shot CLI execution for Windmill orchestration.
//! Follows the Shuttle Protocol:
//! - JSON input (--input or --input-file)
//! - JSON output to stdout
//! - Non-zero exit on failure
//!
//! ## Performance
//!
//! All JSON parsing uses SIMD-accelerated simd-json via ContractEnforcer,
//! providing ~3x faster parsing than serde_json.

use crate::cli::{FatesAction, ManifoldAction, MaintenanceAction, OnboardingAction};
use crate::config::WorkerConfig;
use crate::domains::{fates, maintenance, manifold, onboarding};

use super::{RuntimeError, RuntimeResult, SharedResources};

use std::sync::Arc;
use std::time::Instant;
use tracing::{info, instrument};

/// Step mode runtime
///
/// Executes a single operation and returns JSON output.
/// Designed for Windmill integration with proper error handling.
pub struct StepRuntime {
    resources: Arc<SharedResources>,
}

impl StepRuntime {
    /// Create a new step runtime
    pub async fn new(config: WorkerConfig) -> RuntimeResult<Self> {
        let resources = SharedResources::new(config).await?;

        Ok(Self {
            resources: Arc::new(resources),
        })
    }

    /// Execute a Fates action
    #[instrument(skip(self, input), fields(domain = "fates"))]
    pub async fn execute_fates(&self, action: FatesAction, input: &str) -> RuntimeResult<String> {
        let start = Instant::now();

        // Parse input using SIMD-accelerated JSON (no validation for pipeline steps)
        let input_value: serde_json::Value = self.resources.enforcer.parse_value_str(input)?;

        let result = match action {
            FatesAction::Gate => fates::gate::execute(&self.resources, input_value).await,
            FatesAction::Morta => fates::morta::execute(&self.resources, input_value).await,
            FatesAction::Decima => fates::decima::execute(&self.resources, input_value).await,
            FatesAction::Nona => fates::nona::execute(&self.resources, input_value).await,
            FatesAction::Pipeline => fates::pipeline::execute(&self.resources, input_value).await,
            FatesAction::Evaluate => {
                // Evaluator pattern: returns EvaluationResult with next_step
                fates::gate::execute(&self.resources, input_value).await
            }
        };

        info!(
            duration_ms = start.elapsed().as_millis() as u64,
            "Fates step completed"
        );
        result.map_err(RuntimeError::Domain)
    }

    /// Execute an Onboarding action
    #[instrument(skip(self, input), fields(domain = "onboarding"))]
    pub async fn execute_onboarding(
        &self,
        action: OnboardingAction,
        input: &str,
    ) -> RuntimeResult<String> {
        let start = Instant::now();

        // Parse input using SIMD-accelerated JSON via ContractEnforcer
        let result = match action {
            // =========================================================================
            // Evaluator Pattern Commands (high-level workflow steps)
            // =========================================================================
            OnboardingAction::EvaluateSignup => {
                let input_value: serde_json::Value = self.resources.enforcer.parse_value_str(input)?;
                onboarding::router::execute(&self.resources, input_value).await
            }
            OnboardingAction::EvaluateCreateFamily => {
                let input_value: serde_json::Value = self.resources.enforcer.parse_value_str(input)?;
                onboarding::router::execute(&self.resources, input_value).await
            }
            OnboardingAction::EvaluateAcceptInvitation => {
                let input_value: serde_json::Value = self.resources.enforcer.parse_value_str(input)?;
                onboarding::router::execute(&self.resources, input_value).await
            }
            // Execute variants (perform actual work)
            OnboardingAction::ExecuteSignup => {
                let req: onboarding::SignupRequest = self.resources.enforcer.parse_str(input)?;
                onboarding::signup::execute(&self.resources, req).await
            }
            OnboardingAction::ExecuteCreateFamily => {
                let req: onboarding::CreateFamilyRequest = self.resources.enforcer.parse_str(input)?;
                onboarding::create_family::execute(&self.resources, req).await
            }
            OnboardingAction::ExecuteAcceptInvitation => {
                let req: onboarding::AcceptInvitationRequest = self.resources.enforcer.parse_str(input)?;
                onboarding::accept_invitation::execute(&self.resources, req).await
            }
            OnboardingAction::Process => {
                // Generic JSON for router
                let input_value: serde_json::Value = self.resources.enforcer.parse_value_str(input)?;
                onboarding::router::execute(&self.resources, input_value).await
            }
            
            // =========================================================================
            // Direct Database Operation Commands (Windmill script replacements)
            // These replace inline SQL in Deno scripts with SeaORM-based operations
            // =========================================================================
            OnboardingAction::CheckEmail { email } => {
                onboarding::db_ops::check_email(&self.resources, &email).await
            }
            OnboardingAction::CreateUser { email, name, password_hash } => {
                onboarding::db_ops::create_user(
                    &self.resources,
                    &email,
                    &name,
                    password_hash.as_deref(),
                ).await
            }
            OnboardingAction::CreateSession { user_id } => {
                onboarding::db_ops::create_session(&self.resources, &user_id).await
            }
            OnboardingAction::RecordConsent { user_id, consents_json } => {
                onboarding::db_ops::record_consent(&self.resources, &user_id, &consents_json).await
            }
            OnboardingAction::ValidateInvite { code } => {
                onboarding::db_ops::validate_invite(&self.resources, &code).await
            }
            OnboardingAction::ValidateInvitation { user_id, invite_code } => {
                onboarding::db_ops::validate_invitation(&self.resources, &user_id, &invite_code).await
            }
            OnboardingAction::CheckNeedsFamily { user_id } => {
                onboarding::db_ops::check_needs_family(&self.resources, &user_id).await
            }
            OnboardingAction::CreateTenant { name } => {
                onboarding::db_ops::create_tenant(&self.resources, &name).await
            }
            OnboardingAction::AddMember { tenant_id, user_id, role, name, email } => {
                onboarding::db_ops::add_member(
                    &self.resources,
                    &tenant_id,
                    &user_id,
                    &role,
                    &name,
                    &email,
                ).await
            }
            OnboardingAction::SetPrimaryTenant { user_id, tenant_id } => {
                onboarding::db_ops::set_primary_tenant(&self.resources, &user_id, &tenant_id).await
            }
            OnboardingAction::CreateChannel { tenant_id, channel_type, name, owner_id } => {
                onboarding::db_ops::create_channel(
                    &self.resources,
                    &tenant_id,
                    &channel_type,
                    name.as_deref(),
                    owner_id.as_deref(),
                ).await
            }
            OnboardingAction::IncrementInvite { invitation_id } => {
                onboarding::db_ops::increment_invite(&self.resources, &invitation_id).await
            }
            OnboardingAction::AuditLog { action, user_id, user_email, resource_type, resource_id, metadata_json } => {
                onboarding::db_ops::audit_log(
                    &self.resources,
                    &action,
                    user_id.as_deref(),
                    user_email.as_deref(),
                    resource_type.as_deref(),
                    resource_id.as_deref(),
                    metadata_json.as_deref(),
                ).await
            }
        };

        info!(
            duration_ms = start.elapsed().as_millis() as u64,
            "Onboarding step completed"
        );
        result.map_err(RuntimeError::Domain)
    }

    /// Execute a Manifold action
    #[instrument(skip(self, input), fields(domain = "manifold"))]
    pub async fn execute_manifold(
        &self,
        action: ManifoldAction,
        input: &str,
    ) -> RuntimeResult<String> {
        let start = Instant::now();

        // Parse input using SIMD-accelerated JSON (no validation for pipeline steps)
        let input_value: serde_json::Value = self.resources.enforcer.parse_value_str(input)?;

        let result = match action {
            ManifoldAction::Position => {
                manifold::position::execute(&self.resources, input_value).await
            }
            ManifoldAction::Bind => manifold::bind::execute(&self.resources, input_value).await,
            ManifoldAction::Excite => manifold::excite::execute(&self.resources, input_value).await,
            ManifoldAction::Step => manifold::step::execute(&self.resources, input_value).await,
        };

        info!(
            duration_ms = start.elapsed().as_millis() as u64,
            "Manifold step completed"
        );
        result.map_err(RuntimeError::Domain)
    }

    /// Execute a Maintenance action
    #[instrument(skip(self, input), fields(domain = "maintenance"))]
    pub async fn execute_maintenance(
        &self,
        action: MaintenanceAction,
        input: &str,
    ) -> RuntimeResult<String> {
        let start = Instant::now();

        let result = match action {
            MaintenanceAction::Cleanup => {
                let req: maintenance::CleanupRequest = self.resources.enforcer.parse_str(input)?;
                maintenance::cleanup::execute(&self.resources, req).await
            }
            MaintenanceAction::Health => maintenance::health::execute(&self.resources).await,
            MaintenanceAction::Metrics => maintenance::metrics::execute(&self.resources).await,
        };

        info!(
            duration_ms = start.elapsed().as_millis() as u64,
            "Maintenance step completed"
        );
        result.map_err(RuntimeError::Domain)
    }
}
