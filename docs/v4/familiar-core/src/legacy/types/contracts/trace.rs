//! Trace Types
//!
//! Domain types for UI-safe progress messages.

use serde::{Deserialize, Serialize};

/// Type of trace event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TraceKind {
    /// Overall workflow step
    Step,
    /// Agent thinking/reasoning
    Thought,
    /// Tool invocation
    Tool,
    /// Response token stream
    Token,
    /// Error occurred
    Error,
    /// Metric/timing data
    Metric,
}

/// Status of a trace unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatus {
    /// Step started
    Started,
    /// Step in progress (for long-running)
    InProgress,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was cancelled
    Cancelled,
}

/// Trace payload for UI streaming
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TracePayload {
    /// Monotonic sequence number per-course
    pub seq: u64,
    /// Span identifier
    pub span_id: String,
    /// Parent span (for hierarchy)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    /// Type of trace
    pub kind: TraceKind,
    /// Status of this trace unit
    pub status: TraceStatus,
    /// Agent that produced this trace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Tool name (if kind == Tool)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool arguments (redacted/summarized)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_args: Option<serde_json::Value>,
    /// Tool result (summary only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<String>,
    /// Response tokens (if kind == Token)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<String>,
    /// Duration in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}






