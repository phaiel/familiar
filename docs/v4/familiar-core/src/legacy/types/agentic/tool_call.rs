//! Tool Call Types
//!
//! Types for tool invocation requests and results.

use serde::{Deserialize, Serialize};

/// A request to invoke a tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolCallRequest {
    /// Unique ID for this tool call
    pub call_id: String,
    /// Name of the tool to invoke
    pub tool_name: String,
    /// Arguments to pass to the tool
    pub arguments: serde_json::Value,
}

/// Result from a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolCallResult {
    /// ID of the original call
    pub call_id: String,
    /// Whether the call succeeded
    pub success: bool,
    /// Result data (if successful)
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    #[serde(default)]
    pub error: Option<String>,
}




