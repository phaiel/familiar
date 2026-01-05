//! UI Components
//!
//! Reusable schema-driven UI components.

pub mod block_kit;
pub mod block_kit_builder;

pub use block_kit::{
    BlockMessage, Block, BlockElement, 
    SectionBlock, DividerBlock, ImageBlock, ActionsBlock, ContextBlock, HeaderBlock, InputBlock,
    TextObject, PlainTextObject, MarkdownTextObject, AccordionBlock, AccordionState,
};
pub use block_kit_builder::BlockMessageExt;

