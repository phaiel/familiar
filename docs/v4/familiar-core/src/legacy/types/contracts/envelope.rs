//! Universal Message Envelope (EnvelopeV1)
//!
//! JSON-serializable envelope for building Kafka messages in API services.
//! This is the "convenience" type - the actual wire format uses Protobuf.
//!
//! ## Course-Thread Architecture
//!
//! - `course_id`: The persistent session/history bucket
//! - `shuttle_id`: The transient unit of work (job being processed)
//! - `thread_id`: Reserved for THREAD entity (Person/Concept) - NOT in envelope

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use familiar_primitives::{TenantId, UserId, CourseId, ShuttleId};
use super::{TraceKind, TraceStatus, SignupConsents, RequestContext};

/// Schema version for envelope evolution
pub const ENVELOPE_VERSION: u32 = 1;

// =============================================================================
// EnvelopeV1 - Universal Message Envelope (JSON)
// =============================================================================

/// Universal message envelope for Kafka communication
///
/// This is the JSON-serializable version used by API services.
/// The Kafka producer converts this to Protobuf wire format.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EnvelopeV1 {
    // === Identity ===
    /// Unique message ID (ULID for natural ordering)
    pub message_id: String,
    /// Message type for routing (e.g., "familiar.course.command.start")
    pub message_type: String,
    /// When this message was created
    pub occurred_at: DateTime<Utc>,

    // === Context ===
    /// Tenant (family) context
    pub tenant_id: TenantId,
    /// Course context - persistent session/history bucket
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub course_id: Option<CourseId>,
    /// Shuttle context - transient unit of work (job)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shuttle_id: Option<ShuttleId>,
    /// User who initiated this message
    pub user_id: UserId,

    // === Tracing ===
    /// Correlation ID (ties together workflow, often == course_id)
    pub correlation_id: String,
    /// Causation ID (upstream message that caused this)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,

    // === Producer ===
    /// Information about the message producer
    pub producer: ProducerInfo,

    // === Schema ===
    /// Schema information for validation
    pub schema: SchemaInfo,

    // === Payload (discriminated union) ===
    /// The actual message payload
    pub payload: Payload,
}

/// Producer metadata for tracing and debugging
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProducerInfo {
    /// Service name (e.g., "familiar-api", "familiar-worker")
    pub service: String,
    /// Instance identifier (e.g., pod name, hostname)
    pub instance: String,
    /// Build version (git hash, version tag)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

impl ProducerInfo {
    /// Create producer info for the current service
    pub fn current(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            instance: std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("POD_NAME"))
                .unwrap_or_else(|_| "unknown".to_string()),
            build: option_env!("GIT_HASH").map(String::from),
        }
    }

    /// Create producer info for familiar-api
    pub fn api() -> Self {
        Self::current("familiar-api")
    }

    /// Create producer info for familiar-worker
    pub fn worker() -> Self {
        Self::current("familiar-worker")
    }
}

/// Schema metadata for validation and evolution
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SchemaInfo {
    /// Serialization format ("json", "protobuf", "avro")
    pub format: String,
    /// Schema subject in Schema Registry
    pub subject: String,
    /// Schema version
    pub version: u32,
}

impl Default for SchemaInfo {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            subject: "familiar.envelope.v1".to_string(),
            version: ENVELOPE_VERSION,
        }
    }
}

// =============================================================================
// Payload - Discriminated Union of All Message Types
// =============================================================================

/// Discriminated union of all payload types
///
/// The `type` tag distinguishes between different payloads.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    // =========================================================================
    // Course Domain - Commands
    // =========================================================================

    /// Start a new course from a weave
    CourseStart {
        weave_id: Uuid,
        content: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blocks: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        conversation_history: Vec<serde_json::Value>,
    },

    /// Continue an existing course with additional user input
    CourseContinue {
        user_message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blocks: Option<Vec<serde_json::Value>>,
    },

    /// Cancel a running course
    CourseCancel {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// Retry a failed course
    CourseRetry {
        original_command_id: Uuid,
    },

    // =========================================================================
    // Course Domain - Events
    // =========================================================================

    CourseStarted { weave_id: Uuid },
    CourseSegmented { unit_count: usize },
    CourseClassified {
        entity_types: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        physics_summary: Option<String>,
    },
    CourseCompleted {
        response: String,
        duration_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tokens_used: Option<u32>,
    },
    CourseFailed {
        error: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error_code: Option<String>,
        #[serde(default)]
        retryable: bool,
    },
    CourseCancelled {
        cancelled_by: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    CourseRetrying {
        attempt: u32,
        original_error: String,
    },

    // =========================================================================
    // Onboarding Domain - Commands
    // =========================================================================

    Signup {
        email: String,
        password: String,
        name: String,
        consents: SignupConsents,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        request_context: Option<RequestContext>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        invite_code: Option<String>,
    },

    CreateFamily {
        family_name: String,
    },

    AcceptInvitation {
        invitation_code: String,
    },

    // =========================================================================
    // Onboarding Domain - Events
    // =========================================================================

    SignupCompleted {
        session_token: String,
        needs_family: bool,
    },

    FamilyCreated {
        tenant_name: String,
    },

    InvitationAccepted {
        family_name: String,
    },

    OnboardingFailed {
        error_code: String,
        message: String,
    },

    // =========================================================================
    // Trace - UI-Safe Progress Messages
    // =========================================================================

    Trace {
        seq: u64,
        span_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_span_id: Option<String>,
        kind: TraceKind,
        status: TraceStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent: Option<String>,
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_args: Option<serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_result: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tokens: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
}

impl Payload {
    /// Get the message type name for routing
    pub fn type_name(&self) -> &'static str {
        match self {
            Payload::CourseStart { .. } => "course.command.start",
            Payload::CourseContinue { .. } => "course.command.continue",
            Payload::CourseCancel { .. } => "course.command.cancel",
            Payload::CourseRetry { .. } => "course.command.retry",
            Payload::CourseStarted { .. } => "course.event.started",
            Payload::CourseSegmented { .. } => "course.event.segmented",
            Payload::CourseClassified { .. } => "course.event.classified",
            Payload::CourseCompleted { .. } => "course.event.completed",
            Payload::CourseFailed { .. } => "course.event.failed",
            Payload::CourseCancelled { .. } => "course.event.cancelled",
            Payload::CourseRetrying { .. } => "course.event.retrying",
            Payload::Signup { .. } => "onboarding.command.signup",
            Payload::CreateFamily { .. } => "onboarding.command.create_family",
            Payload::AcceptInvitation { .. } => "onboarding.command.accept_invitation",
            Payload::SignupCompleted { .. } => "onboarding.event.signup_completed",
            Payload::FamilyCreated { .. } => "onboarding.event.family_created",
            Payload::InvitationAccepted { .. } => "onboarding.event.invitation_accepted",
            Payload::OnboardingFailed { .. } => "onboarding.event.failed",
            Payload::Trace { .. } => "trace",
        }
    }

    /// Check if this is a command payload
    pub fn is_command(&self) -> bool {
        matches!(
            self,
            Payload::CourseStart { .. }
                | Payload::CourseContinue { .. }
                | Payload::CourseCancel { .. }
                | Payload::CourseRetry { .. }
                | Payload::Signup { .. }
                | Payload::CreateFamily { .. }
                | Payload::AcceptInvitation { .. }
        )
    }

    /// Check if this is an event payload
    pub fn is_event(&self) -> bool {
        matches!(
            self,
            Payload::CourseStarted { .. }
                | Payload::CourseSegmented { .. }
                | Payload::CourseClassified { .. }
                | Payload::CourseCompleted { .. }
                | Payload::CourseFailed { .. }
                | Payload::CourseCancelled { .. }
                | Payload::CourseRetrying { .. }
                | Payload::SignupCompleted { .. }
                | Payload::FamilyCreated { .. }
                | Payload::InvitationAccepted { .. }
                | Payload::OnboardingFailed { .. }
        )
    }

    /// Check if this is a trace payload
    pub fn is_trace(&self) -> bool {
        matches!(self, Payload::Trace { .. })
    }
}

// =============================================================================
// EnvelopeV1 Implementation
// =============================================================================

impl EnvelopeV1 {
    /// Create a new envelope with the given payload
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        correlation_id: impl Into<String>,
        payload: Payload,
    ) -> Self {
        let message_type = format!("familiar.{}", payload.type_name());
        Self {
            message_id: Ulid::new().to_string(),
            message_type,
            occurred_at: Utc::now(),
            tenant_id,
            course_id: None,
            shuttle_id: None,
            user_id,
            correlation_id: correlation_id.into(),
            causation_id: None,
            producer: ProducerInfo::api(),
            schema: SchemaInfo::default(),
            payload,
        }
    }

    /// Create a command envelope
    pub fn command(
        tenant_id: TenantId,
        user_id: UserId,
        correlation_id: impl Into<String>,
        payload: Payload,
    ) -> Self {
        debug_assert!(payload.is_command(), "payload must be a command");
        let mut envelope = Self::new(tenant_id, user_id, correlation_id, payload);
        envelope.producer = ProducerInfo::api();
        envelope
    }

    /// Create an event envelope (typically from worker)
    pub fn event(
        tenant_id: TenantId,
        user_id: UserId,
        correlation_id: impl Into<String>,
        payload: Payload,
    ) -> Self {
        debug_assert!(payload.is_event(), "payload must be an event");
        let mut envelope = Self::new(tenant_id, user_id, correlation_id, payload);
        envelope.producer = ProducerInfo::worker();
        envelope
    }

    /// Create a trace envelope (typically from worker)
    pub fn trace(
        tenant_id: TenantId,
        user_id: UserId,
        correlation_id: impl Into<String>,
        payload: Payload,
    ) -> Self {
        debug_assert!(payload.is_trace(), "payload must be a trace");
        let mut envelope = Self::new(tenant_id, user_id, correlation_id, payload);
        envelope.producer = ProducerInfo::worker();
        envelope
    }

    /// Set the course ID (persistent session/history bucket)
    pub fn with_course_id(mut self, course_id: CourseId) -> Self {
        self.course_id = Some(course_id);
        self.correlation_id = course_id.to_string();
        self
    }

    /// Set the shuttle ID (transient unit of work)
    pub fn with_shuttle_id(mut self, shuttle_id: ShuttleId) -> Self {
        self.shuttle_id = Some(shuttle_id);
        self
    }

    /// Set the causation ID (upstream message that caused this)
    pub fn with_causation(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    /// Set custom producer info
    pub fn with_producer(mut self, producer: ProducerInfo) -> Self {
        self.producer = producer;
        self
    }

    /// Get the Kafka key for this envelope
    pub fn kafka_key(&self) -> String {
        let session_key = self.course_id
            .map(|c| c.to_string())
            .unwrap_or_else(|| self.correlation_id.clone());
        format!("{}:{}", self.tenant_id, session_key)
    }

    /// Get Kafka headers for this envelope
    pub fn kafka_headers(&self) -> Vec<(&'static str, String)> {
        let mut headers = vec![
            ("message_type", self.message_type.clone()),
            ("tenant_id", self.tenant_id.to_string()),
            ("content_type", "application/json".to_string()),
        ];
        
        if let Some(course_id) = &self.course_id {
            headers.push(("course_id", course_id.to_string()));
        }
        
        if let Some(shuttle_id) = &self.shuttle_id {
            headers.push(("shuttle_id", shuttle_id.to_string()));
        }
        
        headers
    }
}






