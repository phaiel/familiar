//! Fates Pipeline
//!
//! Orchestrates Gate, Morta, Decima, Nona in sequence.
//! Uses SIMD-accelerated JSON parsing via ContractEnforcer.

use crate::runtime::SharedResources;
use super::{gate, morta, decima, nona};
use tracing::{debug, info};

/// Execute full fates pipeline
pub async fn execute(resources: &SharedResources, input: serde_json::Value) -> Result<String, String> {
    debug!("Starting Fates pipeline");

    // 1. Gate: Classification
    let gate_result = gate::execute(resources, input.clone()).await?;
    let gate_output: serde_json::Value = resources.enforcer
        .parse_value_str(&gate_result)
        .map_err(|e| e.to_string())?;

    // 2. Morta: Segmentation
    let morta_result = morta::execute(resources, gate_output.clone()).await?;
    let morta_output: serde_json::Value = resources.enforcer
        .parse_value_str(&morta_result)
        .map_err(|e| e.to_string())?;

    // 3. Decima: Entity extraction
    let decima_result = decima::execute(resources, morta_output.clone()).await?;
    let decima_output: serde_json::Value = resources.enforcer
        .parse_value_str(&decima_result)
        .map_err(|e| e.to_string())?;

    // 4. Nona: Response generation
    let nona_result = nona::execute(resources, decima_output.clone()).await?;
    let nona_output: serde_json::Value = resources.enforcer
        .parse_value_str(&nona_result)
        .map_err(|e| e.to_string())?;

    let result = serde_json::json!({
        "status": "pipeline_complete",
        "stages": {
            "gate": gate_output,
            "morta": morta_output,
            "decima": decima_output,
            "nona": nona_output
        }
    });

    info!("Fates pipeline complete");

    serde_json::to_string(&result).map_err(|e| e.to_string())
}
