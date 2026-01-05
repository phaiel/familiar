//! Block Kit UI Components
//!
//! A schema-first implementation of a Slack-like Block Kit for multimodal messages.
//! This allows rich, structured communication between the API, UI, and Agents.
//!
//! Based on: https://api.slack.com/block-kit

use serde::{Deserialize, Serialize};

/// The root container for a Block Kit message
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BlockMessage {
    pub blocks: Vec<Block>,
}

/// A single visual block in a message
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Section(SectionBlock),
    Divider(DividerBlock),
    Image(ImageBlock),
    Actions(ActionsBlock),
    Context(ContextBlock),
    Header(HeaderBlock),
    Input(InputBlock),
    Accordion(AccordionBlock),
}

// ============================================================================
// Block Types
// ============================================================================

/// Displays text, possibly alongside an accessory (image, button, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SectionBlock {
    pub text: TextObject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<TextObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessory: Option<BlockElement>,
}

/// A visual separator
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DividerBlock {}

/// Displays an image with optional title
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ImageBlock {
    pub image_url: String,
    pub alt_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<PlainTextObject>,
}

/// A container for interactive elements (buttons, selects, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ActionsBlock {
    pub elements: Vec<BlockElement>,
}

/// Displays contextual info (small grey text, images)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ContextBlock {
    pub elements: Vec<ContextElement>,
}

/// A header with large, bold text
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HeaderBlock {
    pub text: PlainTextObject,
}

/// An input field
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct InputBlock {
    pub label: PlainTextObject,
    pub element: BlockElement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<PlainTextObject>,
    #[serde(default)]
    pub optional: bool,
}

/// A collapsible container
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AccordionBlock {
    pub summary: PlainTextObject,
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub initial_state: AccordionState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccordionState {
    #[default]
    Collapsed,
    Expanded,
}

// ============================================================================
// Element Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockElement {
    Button(ButtonElement),
    Image(ImageElement),
    Overflow(OverflowElement),
    PlainTextInput(PlainTextInputElement),
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ButtonElement {
    pub text: PlainTextObject,
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ButtonStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    Primary,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ImageElement {
    pub image_url: String,
    pub alt_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OverflowElement {
    pub action_id: String,
    pub options: Vec<OptionObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PlainTextInputElement {
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<PlainTextObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<String>,
    #[serde(default)]
    pub multiline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextElement {
    Image(ImageElement),
    #[serde(rename = "mrkdwn")]
    Markdown(MarkdownTextObject),
    #[serde(rename = "plain_text")]
    PlainText(PlainTextObject),
}

// ============================================================================
// Composition Objects
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextObject {
    #[serde(rename = "plain_text")]
    Plain(PlainTextObject),
    #[serde(rename = "mrkdwn")]
    Markdown(MarkdownTextObject),
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PlainTextObject {
    pub text: String,
    #[serde(default)]
    pub emoji: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MarkdownTextObject {
    pub text: String,
    #[serde(default)]
    pub verbatim: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OptionObject {
    pub text: PlainTextObject,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<PlainTextObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

// ============================================================================
// Constructors
// ============================================================================

impl BlockMessage {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    pub fn add_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn section(mut self, text: impl Into<String>) -> Self {
        self.blocks.push(Block::Section(SectionBlock {
            text: TextObject::Markdown(MarkdownTextObject {
                text: text.into(),
                verbatim: false,
            }),
            fields: None,
            accessory: None,
        }));
        self
    }

    pub fn header(mut self, text: impl Into<String>) -> Self {
        self.blocks.push(Block::Header(HeaderBlock {
            text: PlainTextObject {
                text: text.into(),
                emoji: true,
            },
        }));
        self
    }

    pub fn divider(mut self) -> Self {
        self.blocks.push(Block::Divider(DividerBlock {}));
        self
    }
}