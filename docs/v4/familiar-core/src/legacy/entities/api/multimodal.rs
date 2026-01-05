//! Multimodal Input Types
//!
//! Schema types for handling non-text inputs (images, audio, documents, webpages).
//! These types are used in WeaveRequest to support multimodal input.

use serde::{Deserialize, Serialize};

/// Input block types for multimodal weave requests
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WeaveBlock {
    Text(TextBlock),
    Image(ImageBlock),
    Audio(AudioBlock),
    Document(DocumentBlock),
    Webpage(WebpageBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TextBlock {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ImageBlock {
    /// base64 data URI or URL
    pub source: String,
    pub alt_text: Option<String>,
    /// Request vision analysis
    #[serde(default = "default_true")]
    pub analyze: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AudioBlock {
    pub source: String,
    pub duration_secs: Option<f64>,
    /// Pre-transcribed (skip Whisper)
    pub transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DocumentBlock {
    pub source: String,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebpageBlock {
    /// URL to scrape
    pub url: String,
    /// Optional CSS selector to extract specific content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Whether to extract images from the page
    #[serde(default)]
    pub extract_images: bool,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Media References (Stored)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum MediaType {
    Image,
    Audio,
    Document,
    Webpage,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MediaRef {
    pub id: String,
    pub media_type: MediaType,
    pub bucket_key: Option<String>,
    pub normalized_text: String,
    pub metadata: serde_json::Value,
}