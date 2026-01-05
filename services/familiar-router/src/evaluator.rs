use crate::{NodeContext, RouterError, RouterResult};
use cel_interpreter::{Context, Program, Value};

/// CEL evaluator for routing decisions
pub struct CALEvaluator {
    context: Context,
}

impl CALEvaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self {
            context: Context::default(),
        }
    }

    /// Evaluate a constraint expression (returns boolean)
    pub fn evaluate_constraint(&self, expression: &str, ctx: &NodeContext) -> RouterResult<bool> {
        let program = Program::compile(expression)
            .map_err(|e| RouterError::InvalidPolicy(format!("CEL compilation: {}", e)))?;

        let cel_context = ctx.to_cel_context();
        let result = program.execute(&self.context, &cel_context)
            .map_err(|e| RouterError::InvalidPolicy(format!("CEL execution: {}", e)))?;

        match result {
            Value::Bool(b) => Ok(b),
            _ => Err(RouterError::InvalidPolicy(
                format!("Constraint '{}' did not evaluate to boolean", expression)
            ))
        }
    }

    /// Evaluate a routing policy expression (returns string)
    pub fn evaluate_routing_policy(&self, expression: &str, ctx: &NodeContext) -> RouterResult<String> {
        let program = Program::compile(expression)
            .map_err(|e| RouterError::InvalidPolicy(format!("CEL compilation: {}", e)))?;

        let cel_context = ctx.to_cel_context();
        let result = program.execute(&self.context, &cel_context)
            .map_err(|e| RouterError::InvalidPolicy(format!("CEL execution: {}", e)))?;

        match result {
            Value::String(s) => Ok(s),
            _ => Err(RouterError::InvalidPolicy(
                format!("Routing policy '{}' did not evaluate to string", expression)
            ))
        }
    }

    /// Evaluate a numeric expression (for metrics and thresholds)
    pub fn evaluate_numeric(&self, expression: &str, ctx: &NodeContext) -> RouterResult<f64> {
        let program = Program::compile(expression)
            .map_err(|e| RouterError::InvalidPolicy(format!("CEL compilation: {}", e)))?;

        let cel_context = ctx.to_cel_context();
        let result = program.execute(&self.context, &cel_context)
            .map_err(|e| RouterError::InvalidPolicy(format!("CEL execution: {}", e)))?;

        match result {
            Value::Int(i) => Ok(i as f64),
            Value::Float(f) => Ok(f),
            _ => Err(RouterError::InvalidPolicy(
                format!("Numeric expression '{}' did not evaluate to number", expression)
            ))
        }
    }
}

impl Default for CALEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_evaluation() {
        let evaluator = CALEvaluator::new();
        let mut ctx = NodeContext::new(semver::Version::parse("1.2.0").unwrap());
        ctx.available_memory = 4 * 1024 * 1024 * 1024; // 4Gi

        assert!(evaluator.evaluate_constraint("available_memory > 2147483648", &ctx).unwrap());
        assert!(!evaluator.evaluate_constraint("available_memory > 8589934592", &ctx).unwrap());
    }

    #[test]
    fn test_routing_policy() {
        let evaluator = CALEvaluator::new();
        let ctx = NodeContext::new(semver::Version::parse("1.2.0").unwrap());

        let result = evaluator.evaluate_routing_policy("'gpu-pool'", &ctx).unwrap();
        assert_eq!(result, "gpu-pool");
    }
}
