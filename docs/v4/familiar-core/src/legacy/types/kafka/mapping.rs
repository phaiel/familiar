//! Wire-to-Domain Type Mapping
//!
//! Provides conversion between wire-level types (from `familiar_contracts`)
//! and domain-level types (from `familiar_core`).
//!
//! ## Design Philosophy
//!
//! Wire types (contracts) use `serde_json::Value` for flexibility and schema evolution.
//! Domain types use concrete types like `WeaveBlock` and `ConversationTurn`.
//!
//! This separation allows:
//! - Contracts to evolve slowly with backward compatibility
//! - Domain types to change freely for internal implementation
//! - Clear boundaries between services

use serde_json::Value as JsonValue;
use crate::WeaveBlock;
use crate::types::agentic::ConversationTurn;

/// Convert domain WeaveBlocks to wire-level JSON values
pub fn weave_blocks_to_wire(blocks: Option<Vec<WeaveBlock>>) -> Option<Vec<JsonValue>> {
    blocks.map(|b| {
        b.into_iter()
            .filter_map(|block| serde_json::to_value(block).ok())
            .collect()
    })
}

/// Convert wire-level JSON values to domain WeaveBlocks
pub fn wire_to_weave_blocks(values: Option<Vec<JsonValue>>) -> Option<Vec<WeaveBlock>> {
    values.map(|v| {
        v.into_iter()
            .filter_map(|val| serde_json::from_value(val).ok())
            .collect()
    })
}

/// Convert domain ConversationTurns to wire-level JSON values
pub fn conversation_to_wire(turns: Vec<ConversationTurn>) -> Vec<JsonValue> {
    turns
        .into_iter()
        .filter_map(|turn| serde_json::to_value(turn).ok())
        .collect()
}

/// Convert wire-level JSON values to domain ConversationTurns
pub fn wire_to_conversation(values: Vec<JsonValue>) -> Vec<ConversationTurn> {
    values
        .into_iter()
        .filter_map(|val| serde_json::from_value(val).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TextBlock;

    #[test]
    fn test_weave_block_roundtrip() {
        let blocks = vec![
            WeaveBlock::Text(TextBlock {
                content: "Hello world".to_string(),
            }),
        ];

        let wire = weave_blocks_to_wire(Some(blocks.clone()));
        let domain = wire_to_weave_blocks(wire);

        assert!(domain.is_some());
        let domain = domain.unwrap();
        assert_eq!(domain.len(), 1);
        match &domain[0] {
            WeaveBlock::Text(t) => assert_eq!(t.content, "Hello world"),
            _ => panic!("Expected TextBlock"),
        }
    }

    #[test]
    fn test_none_passthrough() {
        let wire: Option<Vec<JsonValue>> = weave_blocks_to_wire(None);
        assert!(wire.is_none());

        let domain = wire_to_weave_blocks(None);
        assert!(domain.is_none());
    }
}

