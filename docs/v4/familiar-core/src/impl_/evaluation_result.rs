//! Impl module for evaluation_result types
//!
//! This module contains behavior for generated types.
//! The types are imported from familiar-contracts.

use familiar_contracts::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// Impl blocks for EvaluationResult

// Methods: new, with_data, loom, direct, reject, retry, escalate, complete, skip
impl EvaluationResult { # [doc = " Create a new evaluation result"] pub fn new (next_step : EvaluationStep , reason : impl Into < String >) -> Self { Self { next_step , reason : reason . into () , data : serde_json :: Value :: Null , } } # [doc = " Create with data"] pub fn with_data (next_step : EvaluationStep , reason : impl Into < String > , data : serde_json :: Value) -> Self { Self { next_step , reason : reason . into () , data , } } # [doc = " Process through the Loom"] pub fn loom (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Loom , reason) } # [doc = " Store directly"] pub fn direct (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Direct , reason) } # [doc = " Reject the input"] pub fn reject (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Reject , reason) } # [doc = " Retry the operation"] pub fn retry (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Retry , reason) } # [doc = " Escalate to human review"] pub fn escalate (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Escalate , reason) } # [doc = " Complete the workflow"] pub fn complete (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Complete , reason) } # [doc = " Skip this step"] pub fn skip (reason : impl Into < String >) -> Self { Self :: new (EvaluationStep :: Skip , reason) } }

