//! Evaluation Types - The Evaluator Pattern
//!
//! The Evaluator is the standardized logic gate in the Loom Pattern.
//! This is the "boring infrastructure" approach to workflow branching.
//!
//! Usage:
//! - Minerva CLI commands return EvaluationResult
//! - Windmill reads `next_step` and branches accordingly
//! - The `data` field carries opaque context for the next step

use serde::{Deserialize, Serialize};

// ============================================================================
// EvaluationStep - Typed Enum for Next Actions
// ============================================================================

/// The possible next steps after evaluation
/// 
/// These are the "boring" routing decisions that Windmill uses to branch.
/// By using an enum instead of a string, we get:
/// - Type safety in Rust
/// - Dropdown selection in Windmill
/// - Compile-time validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvaluationStep {
    /// Process through the Loom (AI pipeline)
    Loom,
    /// Store directly without AI processing
    Direct,
    /// Reject the input (validation failed)
    Reject,
    /// Retry the operation (transient failure)
    Retry,
    /// Escalate to human review
    Escalate,
    /// Complete the workflow
    Complete,
    /// Skip this step (no action needed)
    Skip,
}

impl EvaluationStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Loom => "LOOM",
            Self::Direct => "DIRECT",
            Self::Reject => "REJECT",
            Self::Retry => "RETRY",
            Self::Escalate => "ESCALATE",
            Self::Complete => "COMPLETE",
            Self::Skip => "SKIP",
        }
    }
    
    /// Check if this step requires AI processing
    pub fn requires_ai(&self) -> bool {
        matches!(self, Self::Loom | Self::Escalate)
    }
    
    /// Check if this step is terminal (no more processing)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Reject | Self::Skip)
    }
}

impl std::fmt::Display for EvaluationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// EvaluationResult - The Evaluator Output
// ============================================================================

/// Result returned by Evaluator commands
/// 
/// This is the standardized output format for Minerva CLI `evaluate-*` commands.
/// Windmill reads `next_step` to determine which branch to take.
/// 
/// Example:
/// ```json
/// {
///   "next_step": "LOOM",
///   "reason": "User input requires AI classification",
///   "data": { "segments": ["Hello", "world"] }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvaluationResult {
    /// The next step to take (typed enum)
    pub next_step: EvaluationStep,
    /// Human-readable explanation of why this decision was made
    pub reason: String,
    /// Opaque data for the next step (context, extracted fields, etc.)
    pub data: serde_json::Value,
}

impl EvaluationResult {
    /// Create a new evaluation result
    pub fn new(next_step: EvaluationStep, reason: impl Into<String>) -> Self {
        Self {
            next_step,
            reason: reason.into(),
            data: serde_json::Value::Null,
        }
    }
    
    /// Create with data
    pub fn with_data(next_step: EvaluationStep, reason: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            next_step,
            reason: reason.into(),
            data,
        }
    }
    
    /// Process through the Loom
    pub fn loom(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Loom, reason)
    }
    
    /// Store directly
    pub fn direct(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Direct, reason)
    }
    
    /// Reject the input
    pub fn reject(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Reject, reason)
    }
    
    /// Retry the operation
    pub fn retry(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Retry, reason)
    }
    
    /// Escalate to human review
    pub fn escalate(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Escalate, reason)
    }
    
    /// Complete the workflow
    pub fn complete(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Complete, reason)
    }
    
    /// Skip this step
    pub fn skip(reason: impl Into<String>) -> Self {
        Self::new(EvaluationStep::Skip, reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_step_serialization() {
        let step = EvaluationStep::Loom;
        let json = serde_json::to_string(&step).unwrap();
        assert_eq!(json, "\"LOOM\"");
        
        let parsed: EvaluationStep = serde_json::from_str("\"DIRECT\"").unwrap();
        assert_eq!(parsed, EvaluationStep::Direct);
    }

    #[test]
    fn test_evaluation_result_creation() {
        let result = EvaluationResult::loom("Input requires classification");
        assert_eq!(result.next_step, EvaluationStep::Loom);
        assert!(result.next_step.requires_ai());
        
        let result = EvaluationResult::complete("All done");
        assert!(result.next_step.is_terminal());
    }

    #[test]
    fn test_evaluation_result_with_data() {
        let data = serde_json::json!({
            "segments": ["Hello", "world"],
            "confidence": 0.95
        });
        
        let result = EvaluationResult::with_data(
            EvaluationStep::Loom,
            "Segmented input",
            data.clone()
        );
        
        assert_eq!(result.data, data);
    }
}

