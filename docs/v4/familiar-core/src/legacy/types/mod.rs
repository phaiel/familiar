//! Type Definitions
//!
//! Pure type enums and structs. For DAG definitions, see `dags/` module.

// Base types for schema composition
pub mod base;
pub mod api;

// Core entity types
pub mod internal_state_type;
pub mod thread_type;
pub mod relationship_type;
pub mod moment_type;

// Workflow status types
pub mod course_status;
pub mod shuttle_status;
pub mod evaluation;

// Database types
pub mod db_tables;
pub mod db_error;

// Heddle/AI types (used by central DAG definitions)
pub mod heddle_entity_type;
pub mod heddle_response;
pub mod message_role;
pub mod message_intent;
pub mod observer_error;
pub mod segmentation;
pub mod content_classification;
pub mod prompts;
pub mod security;
pub mod ws_protocol;

// Agentic system types (multi-agent orchestration)
pub mod agentic;
pub mod commands;
pub mod ui_agentic;

// Core domain types
pub mod tenant;
pub mod member;

// Conversation types (multi-tenant chat persistence)
pub mod conversation;

// Authentication and onboarding types
pub mod auth;
pub mod onboarding;

// Tool schema library
pub mod tools;

// Kafka message envelopes (Redpanda integration)
pub mod kafka;

// Contract types (domain payloads for opaque envelope)
pub mod contracts;

// Base type re-exports
pub use self::base::{EntityMeta, SystemEntityMeta};

// API type re-exports
pub use self::api::{ApiResult, ApiError, ListResult, SuccessResult};

// Re-exports
pub use self::internal_state_type::InternalStateType;
pub use self::thread_type::ThreadType;
pub use self::relationship_type::RelationshipType;
pub use self::moment_type::MomentType;
pub use self::course_status::CourseStatus;
pub use self::shuttle_status::ShuttleStatus;
pub use self::evaluation::{EvaluationStep, EvaluationResult};
pub use self::db_tables::{DbEntityTable, DbComponentTable};
pub use self::db_error::DbStoreError;

// Heddle/AI re-exports
pub use self::heddle_entity_type::HeddleEntityType;
pub use self::heddle_response::{HeddleResponse, RawWeaveUnit, RawClassification, RawPhysicsHint, RawMessageIntent};
pub use self::message_role::MessageRole;
pub use self::message_intent::{MessageIntent, QueryType, QueryTarget, MessageClassification};
pub use self::observer_error::ObserverError;
pub use self::segmentation::{RawSegment, SegmentationResponse};
pub use self::content_classification::{SegmentClassification, ClassificationPhysics, ContentClassificationResponse};
pub use self::prompts::{PromptConfig, PromptPhase};
pub use self::security::{WINDMILL_SECRETS, WindmillSecrets};
pub use self::ws_protocol::{WsMessage, MessageStatusType};

// Agentic system re-exports
pub use self::agentic::{
    LogLevel, Finding, AgentMessageType, AgentSpeaker, ConversationTurn,
    AgentState, OrchestrationInput, OrchestrationOutput, ToolCallRequest, ToolCallResult,
};
pub use self::commands::{AgenticCommand, AgenticEvent, CommandResult, ConversationHistoryItem};

// UI Agentic types (generated to TypeScript)
pub use self::ui_agentic::{
    ToolCallStatus, UIToolCall, UIThinkingStep,
    UIHeddleResult, UIHeddleSegment, UIClassification, UIPhysicsResult,
    UIThreadItem, UIChannelMessage, UIChannel,
    AgenticFlowResponse,
};

// Core domain re-exports
pub use self::tenant::{Tenant, CreateTenantInput};
pub use self::member::{TenantMember, CreateMemberInput, MemberRole};

// Conversation types (multi-tenant persistence)
pub use self::conversation::{
    // Channel
    Channel, CreateChannelInput, ChannelType, ListChannelsOptions,
    // Message
    Message, CreateMessageInput, MessageRole as DbMessageRole, ConversationMessage, ListMessagesOptions,
    // Entity
    FamiliarEntity, CreateEntityInput, UpdateEntityStatusInput,
    FamiliarEntityType, EntityStatus, EntityPhysics, ListEntitiesOptions,
};

// Auth types (authentication, onboarding, GDPR)
pub use self::auth::{
    // User
    User, PublicUser, CreateUserInput, UpdateUserInput,
    // Session
    AuthSession, CreateSessionInput, SessionCreated,
    // Magic Link
    MagicLink, MagicLinkPurpose, CreateMagicLinkInput, MagicLinkCreated,
    // Invitation
    FamilyInvitation, InviteType, InviteRole, CreateEmailInviteInput, CreateCodeInviteInput, InvitationInfo,
    // Join Request
    JoinRequest, JoinRequestStatus, CreateJoinRequestInput, ReviewJoinRequestInput,
    // GDPR Consent
    ConsentRecord, ConsentType, RecordConsentInput, ConsentStatus,
    // GDPR Export
    DataExportRequest, ExportStatus,
    // GDPR Deletion
    DeletionRequest, DeletionStatus, RequestDeletionInput,
    // Audit
    AuditLogEntry, CreateAuditLogInput,
    // API Types
    SignupRequest, LoginRequest, MagicLinkRequest, AuthResponse, CurrentUser, UserMembership,
};

// Onboarding DAG types (Windmill flows, async tasks, domain events)
pub use self::onboarding::{
    // State machine
    OnboardingState, OnboardingSession,
    // Async tasks
    AsyncTaskStatus, AsyncTask, CreateAsyncTaskInput, AsyncTaskCreated, AsyncTaskPollResponse,
    // Flow inputs/outputs
    SignupConsents, SignupFlowInput, SignupFlowOutput,
    MagicLinkAction, MagicLinkFlowInput, MagicLinkRequestOutput, MagicLinkConsumeOutput,
    CreateFamilyFlowInput, CreateFamilyFlowOutput,
    AcceptInvitationFlowInput, AcceptInvitationFlowOutput,
    // Domain events
    OnboardingEvent, DomainEventEnvelope,
    // Task type constants
    task_types,
};

// Kafka envelope types (Opaque Envelope pattern)
pub use self::kafka::{
    // Trace types
    TraceKind, TraceStatus, TracePayload,
    // Onboarding contracts
    RequestContext,
    SignupRequest as KafkaSignupRequest,
    // Topic names
    topics as kafka_topics,
    SCHEMA_VERSION as KAFKA_SCHEMA_VERSION,
    ENVELOPE_VERSION,
};

// Contract types (domain payloads)
pub use self::contracts::{
    // Course
    CourseStart, CourseContinue, CourseCancel, CourseRetry,
    CourseStarted, CourseSegmented, CourseClassified, CourseCompleted,
    CourseFailed, CourseCancelled, CourseRetrying,
    // Onboarding  
    CreateFamilyRequest, AcceptInvitationRequest,
    SignupCompleted as KafkaSignupCompleted,
    FamilyCreated, InvitationAccepted, OnboardingFailed,
};
