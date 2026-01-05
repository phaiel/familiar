//! Gate - Input Classification and Routing
//!
//! Determines which downstream Fate should handle the message.

use crate::runtime::SharedResources;
use tracing::{debug, info};

/// Execute gate classification
pub async fn execute(resources: &SharedResources, input: serde_json::Value) -> Result<String, String> {
    debug!("Gate processing input");

    // TODO: Implement actual classification using rig-core agent
    let classification = "default";

    let result = serde_json::json!({
        "classification": classification,
        "next_stage": "morta",
        "input": input
    });

    info!(classification = classification, "Gate classification complete");

    serde_json::to_string(&result).map_err(|e| e.to_string())
}
