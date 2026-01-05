//! Evaluator Types (The Evaluator Pattern)
//!
//! The Evaluator is the standardized logic gate in the Loom Pattern.
//! This is the "boring infrastructure" approach to workflow branching.
//!
//! Every evaluate command returns an EvaluationResult that Windmill uses
//! to branch the workflow. The domain data remains an opaque JSON blob.
//!
//! ## The Pattern
//!
//! ```text
//! Windmill Flow
//!     │
//!     ▼
//! minerva onboarding evaluate-signup --input '{"email": "...", ...}'
//!     │
//!     ▼ stdout JSON
//! {
//!     "next_step": "LOOM",
//!     "reason": "Email validated, user needs AI onboarding",
//!     "data": {"email": "...", ...}  // Opaque blob for next step
//! }
//!     │
//!     ▼
//! Windmill branches on next_step:
//!   LOOM     → minerva onboarding execute-signup --input data
//!   DIRECT   → store directly without AI
//!   REJECT   → send rejection email
//!   ESCALATE → queue for manual review
//! ```
//!
//! ## Key Insight
//!
//! Rust evaluates complex data and returns a SIMPLE routing decision.
//! Windmill only sees simple strings for branching.
//! Domain data is opaque JSON passed between steps.

// Re-export from familiar-core for convenience
pub use familiar_core::types::{EvaluationStep, EvaluationResult};

/// Extension trait for worker-specific evaluation helpers
pub trait EvaluationResultExt {
    /// Add correlation context for tracing
    fn with_correlation(self, course_id: &str, shuttle_id: &str) -> Self;
}

impl EvaluationResultExt for EvaluationResult {
    fn with_correlation(mut self, course_id: &str, shuttle_id: &str) -> Self {
        // Add correlation IDs to the data if it's an object
        if let serde_json::Value::Object(ref mut map) = self.data {
            map.insert("_course_id".to_string(), serde_json::Value::String(course_id.to_string()));
            map.insert("_shuttle_id".to_string(), serde_json::Value::String(shuttle_id.to_string()));
        } else if self.data.is_null() {
            self.data = serde_json::json!({
                "_course_id": course_id,
                "_shuttle_id": shuttle_id,
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_result_with_correlation() {
        let result = EvaluationResult::loom("Processing required")
            .with_correlation("course-123", "shuttle-456");
        
        assert_eq!(result.next_step, EvaluationStep::Loom);
        assert_eq!(result.data["_course_id"], "course-123");
        assert_eq!(result.data["_shuttle_id"], "shuttle-456");
    }

    #[test]
    fn test_evaluation_result_serialization() {
        let result = EvaluationResult::with_data(
            EvaluationStep::Direct,
            "Simple storage",
            serde_json::json!({"email": "test@example.com"}),
        );
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"next_step\":\"DIRECT\""));
        assert!(json.contains("\"email\":\"test@example.com\""));
    }
}

