//! Agent Message Types
//!
//! Message type variants for agent responses.

use serde::{Deserialize, Serialize};

use super::log_level::LogLevel;
use super::finding::Finding;

/// Agent message types for different response formats
/// 
/// The agentic system can produce various types of responses depending on
/// what the agent is doing. This enum captures all possible message types.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "message_type", rename_all = "snake_case")]
pub enum AgentMessageType {
    /// A log message from the agent (debugging, status updates)
    Log {
        level: LogLevel,
        content: String,
    },
    /// A question the agent is asking the user
    Question {
        prompt: String,
        #[serde(default)]
        options: Option<Vec<String>>,
    },
    /// An insight derived by the agent
    Insight {
        summary: String,
        confidence: f64,
        #[serde(default)]
        domain: Option<String>,
    },
    /// A detailed analysis result
    Analysis {
        domain: String,
        findings: Vec<Finding>,
    },
    /// A command/action the agent wants to execute
    Command {
        action: String,
        parameters: serde_json::Value,
    },
    /// A simple text response
    Text {
        content: String,
    },
    /// Progress update during long-running tasks
    Progress {
        message: String,
        #[serde(default)]
        percent_complete: Option<f64>,
    },
}




