//! Course Entity - The Persistent Session/History Bucket
//!
//! The Course is the central concept in the Loom Pattern:
//! - Course = persistent session/history bucket (1)
//! - Shuttle = transient unit of work (N per Course)
//! - Thread = THREAD entity (Person/Concept) - protected domain term
//!
//! The Course is the "brain" - it owns the conversation history.
//! The Shuttle is the "muscle" - it does the work for each message.
//!
//! Key architectural principles:
//! 1. Course owns ONLY the history - no transient processing state
//! 2. Course is immutable during Shuttle processing
//! 3. Shuttle commits results to Course history atomically
//! 4. Multiple Shuttles can reference the same Course (1:N)

use serde::{Deserialize, Serialize};

use crate::config::{CourseConfig, TokenEstimationMethod, estimate_tokens};
use crate::primitives::{UUID, Timestamp};
use crate::types::{CourseStatus, MessageRole};

// ============================================================================
// Course Message (History Entry)
// ============================================================================

/// A single message within a Course's history
/// 
/// This is an immutable record of what was said. The Course history
/// is append-only - messages are never modified or deleted.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseMessage {
    /// Unique message ID
    pub id: UUID,
    /// ID of the course this message belongs to
    pub course_id: UUID,
    /// Role of the message sender
    pub role: MessageRole,
    /// Text content of the message
    #[serde(default)]
    pub content: Option<String>,
    /// Which agent sent this message (if role is Assistant)
    #[serde(default)]
    pub agent_speaker: Option<String>,
    /// ID of the Shuttle that processed this message (for tracing)
    #[serde(default)]
    pub shuttle_id: Option<UUID>,
    /// When the message was created
    pub timestamp: Timestamp,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl CourseMessage {
    /// Create a new user message
    pub fn user(course_id: UUID, content: impl Into<String>) -> Self {
        Self {
            id: UUID::new(),
            course_id,
            role: MessageRole::User,
            content: Some(content.into()),
            agent_speaker: None,
            shuttle_id: None,
            timestamp: Timestamp::now(),
            metadata: None,
        }
    }
    
    /// Create a user message with shuttle tracking
    pub fn user_with_shuttle(course_id: UUID, shuttle_id: UUID, content: impl Into<String>) -> Self {
        Self {
            id: UUID::new(),
            course_id,
            role: MessageRole::User,
            content: Some(content.into()),
            agent_speaker: None,
            shuttle_id: Some(shuttle_id),
            timestamp: Timestamp::now(),
            metadata: None,
        }
    }
    
    /// Create a new assistant message
    pub fn assistant(
        course_id: UUID,
        content: impl Into<String>,
        agent: Option<String>,
    ) -> Self {
        Self {
            id: UUID::new(),
            course_id,
            role: MessageRole::Assistant,
            content: Some(content.into()),
            agent_speaker: agent,
            shuttle_id: None,
            timestamp: Timestamp::now(),
            metadata: None,
        }
    }
    
    /// Create an assistant message with shuttle tracking
    pub fn assistant_with_shuttle(
        course_id: UUID,
        shuttle_id: UUID,
        content: impl Into<String>,
        agent: Option<String>,
    ) -> Self {
        Self {
            id: UUID::new(),
            course_id,
            role: MessageRole::Assistant,
            content: Some(content.into()),
            agent_speaker: agent,
            shuttle_id: Some(shuttle_id),
            timestamp: Timestamp::now(),
            metadata: None,
        }
    }
    
    /// Create a system message
    pub fn system(course_id: UUID, content: impl Into<String>) -> Self {
        Self {
            id: UUID::new(),
            course_id,
            role: MessageRole::System,
            content: Some(content.into()),
            agent_speaker: None,
            shuttle_id: None,
            timestamp: Timestamp::now(),
            metadata: None,
        }
    }
}

// ============================================================================
// Course Metadata (Session-Level Info)
// ============================================================================

/// Session-level metadata for a Course
/// 
/// This is for persistent, session-scoped information.
/// Processing metadata (provider, model, latency) belongs on the Shuttle.
#[derive(Debug, Clone, Serialize, Deserialize, Default, schemars::JsonSchema)]
pub struct CourseMetadata {
    /// User's preferred language
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_language: Option<String>,
    
    /// UI theme preference
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_theme: Option<String>,
    
    /// Tags for organization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    
    /// Whether this course is pinned/starred
    #[serde(default)]
    pub is_pinned: bool,
    
    /// Custom user metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<serde_json::Value>,
    
    // ========================================================================
    // Token Tracking (for context pruning)
    // ========================================================================
    
    /// Total tokens in history (cached for fast pruning decisions)
    /// Updated on each commit_message() call
    #[serde(default)]
    pub total_history_tokens: usize,
    
    /// Model-specific tokenizer hint (e.g., "cl100k_base" for Claude/GPT-4)
    /// Used to select the appropriate token counter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer_hint: Option<String>,
}

// ============================================================================
// Course Summary (For List Views)
// ============================================================================

/// Summary of a course for list views
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CourseSummary {
    /// Course ID
    pub id: UUID,
    /// Preview text (truncated first message)
    pub preview: String,
    /// Number of messages in history
    pub message_count: usize,
    /// Current status
    pub status: CourseStatus,
    /// Optional title
    pub title: Option<String>,
    /// Last updated timestamp
    pub updated_at: Timestamp,
}

// ============================================================================
// Course (The Persistent Session/History Bucket)
// ============================================================================

/// A Course is a persistent session tracking conversation history
/// 
/// The Course is the "history bucket" in the Loom Pattern. It:
/// - Owns the conversation history (append-only)
/// - Tracks session status (idle, active, archived)
/// - Survives across Shuttle processing cycles
/// - Has a 1:N relationship with Shuttles
/// 
/// The Course does NOT contain:
/// - Transient processing state (that's on Shuttle)
/// - Current weave/input (that's on Shuttle)
/// - Processing metadata like provider/model (that's on Shuttle)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Course {
    /// Unique identifier for this course
    pub id: UUID,
    
    /// Tenant (family/user) this course belongs to
    pub tenant_id: UUID,
    
    /// Session status (Idle, Active, Archived)
    pub status: CourseStatus,
    
    /// Message history (append-only, immutable during processing)
    #[serde(default)]
    pub history: Vec<CourseMessage>,
    
    /// Session-level metadata
    #[serde(default)]
    pub metadata: CourseMetadata,
    
    /// Optional title for the course
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    /// AI-generated summary
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    
    /// When this course was created
    pub created_at: Timestamp,
    
    /// When this course was last updated (message added)
    pub updated_at: Timestamp,
}

impl Course {
    /// Create a new empty course
    pub fn new(tenant_id: UUID) -> Self {
        let now = Timestamp::now();
        Self {
            id: UUID::new(),
            tenant_id,
            status: CourseStatus::Idle,
            history: vec![],
            metadata: CourseMetadata::default(),
            title: None,
            summary: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a course with an initial title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    // ========================================================================
    // History Management (Append-Only)
    // ========================================================================

    /// Commit a message to history (typically called when Shuttle completes)
    /// 
    /// This is the ONLY way to add messages to the Course.
    /// History is append-only and immutable once committed.
    /// Also updates the cached total_history_tokens count.
    pub fn commit_message(&mut self, message: CourseMessage) {
        // Update token count
        if let Some(content) = &message.content {
            let tokens = estimate_tokens(content, TokenEstimationMethod::Char4);
            self.metadata.total_history_tokens += tokens;
        }
        
        self.history.push(message);
        self.updated_at = Timestamp::now();
        
        // Update status if this is the first message
        if self.history.len() == 1 {
            self.status = CourseStatus::Active;
        }
    }
    
    /// Commit a user message (convenience method)
    pub fn commit_user_message(&mut self, content: impl Into<String>) -> UUID {
        let msg = CourseMessage::user(self.id, content);
        let id = msg.id;
        self.commit_message(msg);
        id
    }
    
    /// Commit a user message with shuttle tracking
    pub fn commit_user_message_with_shuttle(&mut self, shuttle_id: UUID, content: impl Into<String>) -> UUID {
        let msg = CourseMessage::user_with_shuttle(self.id, shuttle_id, content);
        let id = msg.id;
        self.commit_message(msg);
        id
    }
    
    /// Commit an assistant message (convenience method)
    pub fn commit_assistant_message(
        &mut self,
        content: impl Into<String>,
        agent: Option<String>,
    ) -> UUID {
        let msg = CourseMessage::assistant(self.id, content, agent);
        let id = msg.id;
        self.commit_message(msg);
        id
    }
    
    /// Commit an assistant message with shuttle tracking
    pub fn commit_assistant_message_with_shuttle(
        &mut self,
        shuttle_id: UUID,
        content: impl Into<String>,
        agent: Option<String>,
    ) -> UUID {
        let msg = CourseMessage::assistant_with_shuttle(self.id, shuttle_id, content, agent);
        let id = msg.id;
        self.commit_message(msg);
        id
    }
    
    /// Commit a system message (convenience method)
    pub fn commit_system_message(&mut self, content: impl Into<String>) -> UUID {
        let msg = CourseMessage::system(self.id, content);
        let id = msg.id;
        self.commit_message(msg);
        id
    }

    // ========================================================================
    // Status Management
    // ========================================================================
    
    /// Mark course as active (being processed)
    pub fn set_active(&mut self) {
        self.status = CourseStatus::Active;
        self.updated_at = Timestamp::now();
    }
    
    /// Mark course as idle (not being processed)
    pub fn set_idle(&mut self) {
        self.status = CourseStatus::Idle;
        self.updated_at = Timestamp::now();
    }
    
    /// Archive the course
    pub fn archive(&mut self) {
        self.status = CourseStatus::Archived;
        self.updated_at = Timestamp::now();
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the last message in the history
    pub fn last_message(&self) -> Option<&CourseMessage> {
        self.history.last()
    }
    
    /// Get the message count
    pub fn message_count(&self) -> usize {
        self.history.len()
    }
    
    /// Check if course has any messages
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
    
    /// Generate a preview/summary from the first message
    pub fn generate_preview(&self, max_length: usize) -> String {
        self.history
            .first()
            .and_then(|m| m.content.as_ref())
            .map(|c| {
                if c.len() > max_length {
                    format!("{}...", &c[..max_length])
                } else {
                    c.clone()
                }
            })
            .unwrap_or_else(|| "Empty course".to_string())
    }

    /// Create a summary for list views
    pub fn to_summary(&self) -> CourseSummary {
        CourseSummary {
            id: self.id,
            preview: self.generate_preview(100),
            message_count: self.message_count(),
            status: self.status.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
        }
    }
    
    /// Get history for LLM context (as simple role/content pairs)
    /// 
    /// DEPRECATED: Use `get_history_for_context_tokens()` for token-aware pruning.
    pub fn get_history_for_context(&self, limit: usize) -> Vec<(String, String)> {
        self.history
            .iter()
            .rev()
            .take(limit)
            .rev()
            .filter_map(|m| {
                let role = match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                };
                m.content.as_ref().map(|c| (role.to_string(), c.clone()))
            })
            .collect()
    }
    
    /// Get history for LLM context, pruned to fit within token budget
    /// 
    /// This is the preferred method for building LLM context. It:
    /// 1. Iterates from most recent to oldest messages
    /// 2. Accumulates tokens until the budget is exceeded
    /// 3. Always includes at least `min_messages` for coherence
    /// 4. Returns messages in chronological order
    /// 
    /// # Arguments
    /// * `config` - Course configuration with token limits
    /// 
    /// # Returns
    /// Vec of (role, content) pairs, oldest first
    pub fn get_history_for_context_tokens(&self, config: &CourseConfig) -> Vec<(String, String)> {
        let max_tokens = config.available_tokens();
        let min_messages = config.min_messages;
        let method = config.estimation_method;
        
        let mut result = Vec::new();
        let mut token_count = 0;
        let mut messages_included = 0;
        
        // Iterate from most recent to oldest
        for msg in self.history.iter().rev() {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            };
            
            if let Some(content) = &msg.content {
                let msg_tokens = estimate_tokens(content, method);
                
                // Always include min_messages, otherwise check token budget
                let should_include = messages_included < min_messages 
                    || token_count + msg_tokens <= max_tokens;
                
                if should_include {
                    token_count += msg_tokens;
                    messages_included += 1;
                    result.push((role.to_string(), content.clone()));
                } else {
                    // Would exceed budget and we have enough messages
                    break;
                }
            }
        }
        
        // Reverse to get chronological order
        result.reverse();
        result
    }
    
    /// Get history with detailed token information for debugging
    pub fn get_history_with_token_info(&self, config: &CourseConfig) -> ContextWindowInfo {
        let max_tokens = config.available_tokens();
        let method = config.estimation_method;
        
        let mut messages = Vec::new();
        let mut total_tokens = 0;
        
        for msg in &self.history {
            if let Some(content) = &msg.content {
                let tokens = estimate_tokens(content, method);
                total_tokens += tokens;
                messages.push(MessageTokenInfo {
                    id: msg.id,
                    role: msg.role.clone(),
                    tokens,
                    included: total_tokens <= max_tokens,
                });
            }
        }
        
        ContextWindowInfo {
            total_messages: self.history.len(),
            total_tokens,
            max_tokens,
            messages_included: messages.iter().filter(|m| m.included).count(),
            tokens_used: messages.iter().filter(|m| m.included).map(|m| m.tokens).sum(),
            messages,
        }
    }
    
    /// Recalculate total_history_tokens from scratch
    /// 
    /// Call this if you suspect the cached token count is out of sync,
    /// or after loading a Course from the database.
    pub fn recalculate_token_count(&mut self) {
        self.metadata.total_history_tokens = self.history
            .iter()
            .filter_map(|m| m.content.as_ref())
            .map(|c| estimate_tokens(c, TokenEstimationMethod::Char4))
            .sum();
    }
}

/// Token information for a single message
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MessageTokenInfo {
    pub id: UUID,
    pub role: MessageRole,
    pub tokens: usize,
    pub included: bool,
}

/// Summary of context window usage
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ContextWindowInfo {
    pub total_messages: usize,
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub messages_included: usize,
    pub tokens_used: usize,
    pub messages: Vec<MessageTokenInfo>,
}

impl From<&Course> for CourseSummary {
    fn from(course: &Course) -> Self {
        course.to_summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_creation() {
        let tenant = UUID::new();
        let course = Course::new(tenant).with_title("Test Course");
        
        assert_eq!(course.tenant_id, tenant);
        assert_eq!(course.status, CourseStatus::Idle);
        assert!(course.is_empty());
        assert_eq!(course.title, Some("Test Course".to_string()));
    }

    #[test]
    fn test_course_history_commit() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        // Commit a user message
        let msg_id = course.commit_user_message("Hello!");
        assert_eq!(course.message_count(), 1);
        assert_eq!(course.status, CourseStatus::Active); // Auto-activated
        assert_eq!(course.history[0].id, msg_id);
        assert_eq!(course.history[0].role, MessageRole::User);
        
        // Commit an assistant response
        course.commit_assistant_message("Hi there!", Some("concierge".to_string()));
        assert_eq!(course.message_count(), 2);
        assert_eq!(course.history[1].role, MessageRole::Assistant);
        assert_eq!(course.history[1].agent_speaker, Some("concierge".to_string()));
    }

    #[test]
    fn test_course_with_shuttle_tracking() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        let shuttle_id = UUID::new();
        
        // Commit messages with shuttle tracking
        course.commit_user_message_with_shuttle(shuttle_id, "Hello!");
        course.commit_assistant_message_with_shuttle(shuttle_id, "Hi!", None);
        
        // Both messages should have the shuttle_id
        assert_eq!(course.history[0].shuttle_id, Some(shuttle_id));
        assert_eq!(course.history[1].shuttle_id, Some(shuttle_id));
    }

    #[test]
    fn test_course_summary() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        course.commit_user_message("This is a long message that should be truncated in the preview");
        
        let summary = course.to_summary();
        assert!(summary.preview.len() <= 103); // 100 + "..."
        assert_eq!(summary.message_count, 1);
    }
    
    #[test]
    fn test_get_history_for_context() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        course.commit_user_message("Hello");
        course.commit_assistant_message("Hi!", None);
        course.commit_user_message("How are you?");
        course.commit_assistant_message("I'm doing well!", None);
        
        let context = course.get_history_for_context(2);
        assert_eq!(context.len(), 2);
        assert_eq!(context[0].0, "user");
        assert_eq!(context[0].1, "How are you?");
        assert_eq!(context[1].0, "assistant");
        assert_eq!(context[1].1, "I'm doing well!");
    }
    
    #[test]
    fn test_token_counting_on_commit() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        // Token count should start at 0
        assert_eq!(course.metadata.total_history_tokens, 0);
        
        // "Hello" = 5 chars -> ~2 tokens (char4 method: (5+3)/4 = 2)
        course.commit_user_message("Hello");
        assert!(course.metadata.total_history_tokens > 0);
        
        let tokens_after_first = course.metadata.total_history_tokens;
        
        // Add another message - tokens should increase
        course.commit_assistant_message("Hi there!", None);
        assert!(course.metadata.total_history_tokens > tokens_after_first);
    }
    
    #[test]
    fn test_get_history_for_context_tokens() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        // Add some messages
        course.commit_user_message("Hello");
        course.commit_assistant_message("Hi!", None);
        course.commit_user_message("How are you today?");
        course.commit_assistant_message("I'm doing well, thank you for asking!", None);
        
        // With large token budget, all messages should be included
        let config = CourseConfig {
            max_context_tokens: 10000,
            reserved_tokens: 1000,
            min_messages: 2,
            estimation_method: TokenEstimationMethod::Char4,
        };
        
        let context = course.get_history_for_context_tokens(&config);
        assert_eq!(context.len(), 4);
        assert_eq!(context[0].0, "user");
        assert_eq!(context[0].1, "Hello");
    }
    
    #[test]
    fn test_get_history_for_context_tokens_with_small_budget() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        // Add messages with varying lengths
        course.commit_user_message("Short");
        course.commit_assistant_message("Also short", None);
        course.commit_user_message("This is a much longer message that will take more tokens");
        course.commit_assistant_message("And this is an even longer response that definitely takes many tokens in the context window", None);
        
        // With tiny token budget, should only get min_messages
        let config = CourseConfig {
            max_context_tokens: 20, // Very small
            reserved_tokens: 5,
            min_messages: 2,
            estimation_method: TokenEstimationMethod::Char4,
        };
        
        let context = course.get_history_for_context_tokens(&config);
        
        // Should have at least min_messages
        assert!(context.len() >= 2);
        
        // Most recent messages should be included first
        let last_msg = context.last().unwrap();
        assert_eq!(last_msg.0, "assistant");
    }
    
    #[test]
    fn test_recalculate_token_count() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        course.commit_user_message("Hello");
        course.commit_assistant_message("Hi!", None);
        
        let original_count = course.metadata.total_history_tokens;
        
        // Manually corrupt the count
        course.metadata.total_history_tokens = 0;
        
        // Recalculate should restore it
        course.recalculate_token_count();
        assert_eq!(course.metadata.total_history_tokens, original_count);
    }
    
    #[test]
    fn test_context_window_info() {
        let tenant = UUID::new();
        let mut course = Course::new(tenant);
        
        course.commit_user_message("Hello");
        course.commit_assistant_message("Hi there!", None);
        
        let config = CourseConfig::default();
        let info = course.get_history_with_token_info(&config);
        
        assert_eq!(info.total_messages, 2);
        assert!(info.total_tokens > 0);
        assert_eq!(info.messages.len(), 2);
        
        // With default config (8000 tokens), all should be included
        assert_eq!(info.messages_included, 2);
    }
}
