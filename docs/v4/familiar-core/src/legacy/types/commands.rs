//! Command and Event Types for Service Communication
//!
//! These types implement the event/command pattern for communication between
//! the API layer and backend services. Designed to work with direct calls today
//! and message brokers (Redpanda) in the future.
//!
//! Course-Thread Architecture:
//! - course_id: The persistent session/history bucket
//! - thread_id: Reserved for THREAD entity (Person/Concept) - NOT used here
//!
//! Commands: Actions requested by clients (API → Services)
//! Events: Notifications emitted by services (Services → Clients)

use serde::{Deserialize, Serialize};

use crate::WeaveBlock;
use super::agentic::{AgentMessageType, AgentState};

// ============================================================================
// Commands (API → Services)
// ============================================================================

/// A single item in conversation history
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConversationHistoryItem {
    /// Role: "user" or "assistant"
    pub role: String,
    /// Content of the message
    pub content: String,
}

/// Commands that can be sent to the agentic system
/// 
/// These commands are the primary interface for interacting with the
/// multi-agent orchestration system. They can be sent directly via HTTP
/// or published to a message broker.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "command_type", rename_all = "snake_case")]
pub enum AgenticCommand {
    /// Send a new message to the agentic system
    SendMessage {
        /// Course ID (None = start new course/session)
        #[serde(default)]
        course_id: Option<String>,
        /// Text content of the message
        content: String,
        /// Conversation history for context
        #[serde(default)]
        conversation_history: Option<Vec<ConversationHistoryItem>>,
        /// Optional multimodal blocks (images, audio, etc.)
        #[serde(default)]
        blocks: Option<Vec<WeaveBlock>>,
        /// Request ID for tracing
        request_id: String,
        /// Tenant ID
        tenant_id: String,
        /// Override the default flow path (e.g., "u/phaiel/loom")
        #[serde(default)]
        flow_path: Option<String>,
    },
    /// Continue processing in an existing course
    ContinueCourse {
        /// ID of the course to continue
        course_id: String,
        /// Request ID for tracing
        request_id: String,
    },
    /// Cancel a running task
    CancelTask {
        /// ID of the task to cancel
        task_id: String,
        /// Reason for cancellation
        #[serde(default)]
        reason: Option<String>,
    },
    /// Request course history
    GetCourseHistory {
        /// ID of the course
        course_id: String,
        /// Maximum number of messages to return
        #[serde(default)]
        limit: Option<usize>,
    },
}

impl AgenticCommand {
    /// Create a new SendMessage command
    pub fn send_message(
        tenant_id: impl Into<String>,
        content: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self::SendMessage {
            course_id: None,
            content: content.into(),
            conversation_history: None,
            blocks: None,
            request_id: request_id.into(),
            tenant_id: tenant_id.into(),
            flow_path: None,
        }
    }
    
    /// Create a SendMessage command for an existing course
    pub fn reply_to_course(
        tenant_id: impl Into<String>,
        course_id: impl Into<String>,
        content: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self::SendMessage {
            course_id: Some(course_id.into()),
            content: content.into(),
            conversation_history: None,
            blocks: None,
            request_id: request_id.into(),
            tenant_id: tenant_id.into(),
            flow_path: None,
        }
    }
    
    /// Create a SendMessage command with conversation history
    pub fn send_with_context(
        tenant_id: impl Into<String>,
        content: impl Into<String>,
        conversation_history: Vec<ConversationHistoryItem>,
        request_id: impl Into<String>,
    ) -> Self {
        Self::SendMessage {
            course_id: None,
            content: content.into(),
            conversation_history: Some(conversation_history),
            blocks: None,
            request_id: request_id.into(),
            tenant_id: tenant_id.into(),
            flow_path: None,
        }
    }
    
    /// Create a SendMessage command with a specific flow path
    pub fn send_to_flow(
        tenant_id: impl Into<String>,
        content: impl Into<String>,
        request_id: impl Into<String>,
        flow_path: impl Into<String>,
    ) -> Self {
        Self::SendMessage {
            course_id: None,
            content: content.into(),
            conversation_history: None,
            blocks: None,
            request_id: request_id.into(),
            tenant_id: tenant_id.into(),
            flow_path: Some(flow_path.into()),
        }
    }
}

// ============================================================================
// Events (Services → Clients)
// ============================================================================

/// Events emitted by the agentic system
/// 
/// These events notify clients about state changes and results from the
/// multi-agent system. They can be delivered via WebSocket, SSE, or
/// consumed from a message broker.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AgenticEvent {
    /// A new message was received/generated
    MessageReceived {
        /// Course ID (session)
        course_id: String,
        /// Message ID
        message_id: String,
        /// Role (user, assistant, system)
        role: String,
        /// Message content
        #[serde(default)]
        content: Option<String>,
        /// Structured message type
        #[serde(default)]
        message_type: Option<AgentMessageType>,
        /// Which agent sent this
        #[serde(default)]
        agent: Option<String>,
        /// Timestamp
        timestamp: String,
    },
    /// An agent is now speaking/processing
    AgentSpeaking {
        /// Course ID
        course_id: String,
        /// Which agent is speaking
        agent: String,
        /// Optional status message
        #[serde(default)]
        status: Option<String>,
    },
    /// A task was completed successfully
    TaskCompleted {
        /// Course ID
        course_id: String,
        /// Task ID (same as request_id)
        task_id: String,
        /// Result data
        result: serde_json::Value,
    },
    /// An error occurred
    Error {
        /// Course ID (if applicable)
        #[serde(default)]
        course_id: Option<String>,
        /// Error code
        code: String,
        /// Error message
        error: String,
        /// Additional details
        #[serde(default)]
        details: Option<serde_json::Value>,
    },
    /// Course state was updated
    CourseStateChanged {
        /// Course ID
        course_id: String,
        /// Updated agent state
        state: AgentState,
    },
    /// Progress update for long-running operations
    Progress {
        /// Course ID
        course_id: String,
        /// Task ID
        task_id: String,
        /// Progress message
        message: String,
        /// Percent complete (0-100)
        #[serde(default)]
        percent: Option<f64>,
    },
}

impl AgenticEvent {
    /// Create a MessageReceived event
    pub fn message_received(
        course_id: impl Into<String>,
        message_id: impl Into<String>,
        role: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::MessageReceived {
            course_id: course_id.into(),
            message_id: message_id.into(),
            role: role.into(),
            content: Some(content.into()),
            message_type: None,
            agent: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Create an Error event
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            course_id: None,
            code: code.into(),
            error: message.into(),
            details: None,
        }
    }
    
    /// Create an Error event for a specific course
    pub fn course_error(
        course_id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Error {
            course_id: Some(course_id.into()),
            code: code.into(),
            error: message.into(),
            details: None,
        }
    }
}

// ============================================================================
// Command Result
// ============================================================================

/// Result of processing a command
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CommandResult {
    /// Whether the command was accepted
    pub accepted: bool,
    /// Job ID for async tracking
    #[serde(default)]
    pub job_id: Option<String>,
    /// Course ID (for SendMessage commands)
    #[serde(default)]
    pub course_id: Option<String>,
    /// WebSocket URL for streaming events
    #[serde(default)]
    pub ws_url: Option<String>,
    /// Error message if not accepted
    #[serde(default)]
    pub error: Option<String>,
}

impl CommandResult {
    /// Create a successful result
    pub fn success(job_id: impl Into<String>, course_id: impl Into<String>) -> Self {
        let job = job_id.into();
        Self {
            accepted: true,
            job_id: Some(job.clone()),
            course_id: Some(course_id.into()),
            ws_url: Some(format!("/api/courses/{}/ws", job)),
            error: None,
        }
    }
    
    /// Create an error result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            accepted: false,
            job_id: None,
            course_id: None,
            ws_url: None,
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = AgenticCommand::send_message("tenant-123", "Hello!", "req-456");
        let json = serde_json::to_string(&cmd).unwrap();
        
        assert!(json.contains("command_type"));
        assert!(json.contains("send_message"));
        assert!(json.contains("tenant-123"));
    }

    #[test]
    fn test_event_serialization() {
        let event = AgenticEvent::message_received("course-1", "msg-1", "assistant", "Hello!");
        let json = serde_json::to_string(&event).unwrap();
        
        assert!(json.contains("event_type"));
        assert!(json.contains("message_received"));
    }

    #[test]
    fn test_command_result() {
        let result = CommandResult::success("job-123", "course-456");
        assert!(result.accepted);
        assert_eq!(result.ws_url, Some("/api/courses/job-123/ws".to_string()));
    }
}
