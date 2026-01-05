//! Block Kit Builder
//!
//! Converts domain responses (CourseResponse, etc.) into Block Kit messages.
//! Schema-first, ECS-inspired: Messages are entities, blocks are components.

use crate::components::ui::block_kit::{
    BlockMessage, Block, SectionBlock, DividerBlock, HeaderBlock, AccordionBlock,
    TextObject, PlainTextObject, MarkdownTextObject, BlockElement, ButtonElement,
    AccordionState,
};
use crate::CourseResponse;

/// Extension trait for BlockMessage to add builder methods
pub trait BlockMessageExt {
    /// Convert a CourseResponse into a Block Kit message
    fn from_course_response(response: &CourseResponse) -> Self;
    /// Create a typing indicator message
    fn typing_indicator(message: Option<String>) -> Self;
    /// Create a progress message
    fn progress(status: &str, details: Option<&str>) -> Self;
    /// Create an error message
    fn error(message: &str) -> Self;
}

impl BlockMessageExt for BlockMessage {
    /// Convert a CourseResponse into a Block Kit message
    fn from_course_response(response: &CourseResponse) -> Self {
        let mut blocks = Vec::new();

        // Header
        blocks.push(Block::Header(HeaderBlock {
            text: PlainTextObject {
                text: "🌀 Windmill Pipeline".to_string(),
                emoji: true,
            },
        }));

        // Original weave
        if !response.original_weave.is_empty() {
            blocks.push(Block::Section(SectionBlock {
                text: TextObject::Markdown(MarkdownTextObject {
                    text: format!("**Original:** {}", response.original_weave),
                    verbatim: false,
                }),
                fields: None,
                accessory: None,
            }));
        }

        blocks.push(Block::Divider(DividerBlock {}));

        // Weave units
        if !response.weave_units.is_empty() {
            blocks.push(Block::Header(HeaderBlock {
                text: PlainTextObject {
                    text: format!("{} Weave Unit{}", response.unit_count, if response.unit_count != 1 { "s" } else { "" }),
                    emoji: false,
                },
            }));

            for unit in &response.weave_units {
                let purpose_badge = unit.purpose.as_ref()
                    .map(|p| {
                        let icon = match p.as_str() {
                            "LOG" => "📝",
                            "QUERY" => "❓",
                            "COMMAND" => "⚡",
                            "INFER" => "🔮",
                            "REFERENCE" => "🔍",
                            _ => "❔",
                        };
                        format!("`{} {}`", icon, p)
                    })
                    .unwrap_or_default();

                // Show ALL classifications with probabilities (sorted by probability, highest first)
                // Probabilities are not binary - a weave_unit may spawn multiple entity types
                // Classification system just classifies, doesn't make decisions
                let classifications = if !unit.classifications.is_empty() {
                    let mut sorted: Vec<_> = unit.classifications.iter().collect();
                    sorted.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
                    
                    format!(
                        "**Classifications:** {}\n",
                        sorted
                            .iter()
                            .map(|c| format!("`{}: {:.1}%`", c.entity_type, c.weight * 100.0))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                } else {
                    String::new()
                };

                let spawned_info = if !unit.spawned_ids.is_empty() {
                    format!("✅ Spawned: {}", unit.spawned_ids.len())
                } else if unit.purpose.as_deref() == Some("LOG") {
                    "⏸️ No spawn (below threshold)".to_string()
                } else {
                    String::new()
                };

                let mut unit_text = format!("**Unit {}:** {}\n", unit.index + 1, unit.content);
                if !purpose_badge.is_empty() {
                    unit_text.push_str(&format!("{}\n", purpose_badge));
                }
                if !classifications.is_empty() {
                    unit_text.push_str(&format!("{}\n", classifications));
                }
                if !spawned_info.is_empty() {
                    unit_text.push_str(&format!("{}\n", spawned_info));
                }

                blocks.push(Block::Section(SectionBlock {
                    text: TextObject::Markdown(MarkdownTextObject {
                        text: unit_text.trim().to_string(),
                        verbatim: false,
                    }),
                    fields: None,
                    accessory: None,
                }));
            }
        }

        // Spawned entities (in accordion)
        if let Some(entities) = &response.entities {
            if !entities.is_empty() {
                let entity_blocks: Vec<Block> = entities
                    .iter()
                    .map(|e| {
                        Block::Section(SectionBlock {
                            text: TextObject::Markdown(MarkdownTextObject {
                                text: format!(
                                    "**{}** `{}`\n{}",
                                    e.entity_type,
                                    &e.id[..8],
                                    e.content_preview
                                ),
                                verbatim: false,
                            }),
                            fields: None,
                            accessory: None,
                        })
                    })
                    .collect();

                blocks.push(Block::Accordion(AccordionBlock {
                    summary: PlainTextObject {
                        text: format!("{} Entit{} Spawned", entities.len(), if entities.len() != 1 { "ies" } else { "y" }),
                        emoji: false,
                    },
                    blocks: entity_blocks,
                    initial_state: AccordionState::Collapsed,
                }));
            }
        }

        // Debug JSONs (in accordion)
        let mut debug_blocks = Vec::new();
        if let Some(req) = &response.debug_llm_request {
            debug_blocks.push(Block::Section(SectionBlock {
                text: TextObject::Markdown(MarkdownTextObject {
                    text: format!("**System Prompt:**\n```\n{}\n```", &req.system_prompt),
                    verbatim: false,
                }),
                fields: None,
                accessory: Some(BlockElement::Button(ButtonElement {
                    text: PlainTextObject {
                        text: "📋 Copy".to_string(),
                        emoji: false,
                    },
                    action_id: format!("copy-request-{}", response.course_id),
                    url: None,
                    value: Some(serde_json::to_string(req).unwrap_or_default()),
                    style: None,
                })),
            }));
        }
        if let Some(resp) = &response.debug_llm_response {
            debug_blocks.push(Block::Section(SectionBlock {
                text: TextObject::Markdown(MarkdownTextObject {
                    text: format!("**LLM Response:**\n```\n{}\n```", resp),
                    verbatim: false,
                }),
                fields: None,
                accessory: Some(BlockElement::Button(ButtonElement {
                    text: PlainTextObject {
                        text: "📋 Copy".to_string(),
                        emoji: false,
                    },
                    action_id: format!("copy-response-{}", response.course_id),
                    url: None,
                    value: Some(resp.clone()),
                    style: None,
                })),
            }));
        }
        if !debug_blocks.is_empty() {
            blocks.push(Block::Accordion(AccordionBlock {
                summary: PlainTextObject {
                    text: "🔍 Debug Information".to_string(),
                    emoji: false,
                },
                blocks: debug_blocks,
                initial_state: AccordionState::Collapsed,
            }));
        }

        Self { blocks }
    }

    /// Create a typing indicator message
    fn typing_indicator(message: Option<String>) -> Self {
        Self {
            blocks: vec![Block::Section(SectionBlock {
                text: TextObject::Markdown(MarkdownTextObject {
                    text: format!("⏳ {}", message.unwrap_or_else(|| "Processing...".to_string())),
                    verbatim: false,
                }),
                fields: None,
                accessory: None,
            })],
        }
    }

    /// Create a progress message
    fn progress(status: &str, details: Option<&str>) -> Self {
        let mut text = format!("🔄 {}", status);
        if let Some(d) = details {
            text.push_str(&format!("\n{}", d));
        }
        Self {
            blocks: vec![Block::Section(SectionBlock {
                text: TextObject::Markdown(MarkdownTextObject {
                    text,
                    verbatim: false,
                }),
                fields: None,
                accessory: None,
            })],
        }
    }

    /// Create an error message
    fn error(message: &str) -> Self {
        Self {
            blocks: vec![Block::Section(SectionBlock {
                text: TextObject::Markdown(MarkdownTextObject {
                    text: format!("❌ **Error:** {}", message),
                    verbatim: false,
                }),
                fields: None,
                accessory: None,
            })],
        }
    }
}

