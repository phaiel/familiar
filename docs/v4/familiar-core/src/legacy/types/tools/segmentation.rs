//! Multi-Modal Segmentation Tool Schemas
//!
//! Tools for breaking input into semantic units across different modalities:
//! - Text: Topic shifts, temporal markers, entity mentions
//! - Audio: Speaker changes, topic shifts, emotional tone changes
//! - Vision: Scene descriptions, objects, actions, emotional content
//! - Video: Combined vision + audio + temporal sequence

use serde::{Deserialize, Serialize};

use super::base::{Modality, ModalityInput};

// ============================================================================
// Segmentation Tool Input/Output
// ============================================================================

/// Input for segmentation tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentationInput {
    /// The input to segment
    pub input: ModalityInput,
    /// Context from conversation/session
    #[serde(default)]
    pub context: Option<SegmentationContext>,
    /// Configuration options
    #[serde(default)]
    pub config: SegmentationConfig,
}

/// Context for segmentation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentationContext {
    /// Previous segments in the conversation
    #[serde(default)]
    pub previous_segments: Vec<Segment>,
    /// Known entities for reference resolution
    #[serde(default)]
    pub known_entities: Vec<KnownEntity>,
    /// Current speaker (if known)
    #[serde(default)]
    pub current_speaker: Option<String>,
    /// Tenant ID for multi-tenancy
    pub tenant_id: String,
}

/// Configuration for segmentation behavior
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentationConfig {
    /// Maximum number of segments to produce
    #[serde(default = "default_max_segments")]
    pub max_segments: usize,
    /// Minimum segment length (chars for text, seconds for audio/video)
    #[serde(default)]
    pub min_segment_length: Option<f64>,
    /// Whether to extract entity mentions
    #[serde(default = "default_true")]
    pub extract_entities: bool,
    /// Whether to extract temporal markers
    #[serde(default = "default_true")]
    pub extract_temporal: bool,
    /// Language hint (ISO 639-1)
    #[serde(default)]
    pub language: Option<String>,
}

fn default_max_segments() -> usize {
    20
}

fn default_true() -> bool {
    true
}

impl Default for SegmentationConfig {
    fn default() -> Self {
        Self {
            max_segments: default_max_segments(),
            min_segment_length: None,
            extract_entities: true,
            extract_temporal: true,
            language: None,
        }
    }
}

/// Known entity for reference resolution
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct KnownEntity {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub entity_type: String,
}

// ============================================================================
// Segment Types
// ============================================================================

/// A segmented unit of content
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Segment {
    /// Unique segment ID
    pub id: String,
    /// Index in the sequence
    pub index: usize,
    /// The segmented content
    pub content: String,
    /// Original modality
    pub modality: Modality,
    /// Segment boundaries
    pub boundaries: SegmentBoundaries,
    /// Extracted features
    pub features: SegmentFeatures,
    /// Confidence in segmentation decision
    pub confidence: f64,
}

/// Boundaries of a segment within the original input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentBoundaries {
    /// Start position (char index for text, milliseconds for audio/video)
    pub start: u64,
    /// End position
    pub end: u64,
    /// For audio/video: speaker ID if detected
    #[serde(default)]
    pub speaker_id: Option<String>,
}

/// Extracted features from a segment
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentFeatures {
    /// Primary subject/actor mentioned
    #[serde(default)]
    pub subject: Option<String>,
    /// Entity mentions found
    #[serde(default)]
    pub mentions: Vec<EntityMention>,
    /// Temporal markers detected
    #[serde(default)]
    pub temporal_markers: Vec<TemporalMarker>,
    /// Emotional tone (if detectable)
    #[serde(default)]
    pub emotional_tone: Option<EmotionalTone>,
    /// Keywords extracted
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Language detected
    #[serde(default)]
    pub language: Option<String>,
}

/// An entity mention within a segment
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EntityMention {
    /// The text of the mention
    pub text: String,
    /// Position in segment
    pub start: usize,
    pub end: usize,
    /// Type of entity mentioned
    pub entity_type: EntityMentionType,
    /// Resolved entity ID (if matched to known entity)
    #[serde(default)]
    pub resolved_id: Option<String>,
    /// Confidence in detection
    pub confidence: f64,
}

/// Types of entity mentions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityMentionType {
    Person,
    Place,
    Organization,
    DateTime,
    Event,
    Concept,
    Object,
    Activity,
}

/// A temporal marker in the segment
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TemporalMarker {
    /// The text of the marker
    pub text: String,
    /// Parsed temporal value (if parseable)
    #[serde(default)]
    pub parsed: Option<ParsedTemporal>,
    /// Type of temporal reference
    pub marker_type: TemporalMarkerType,
}

/// Parsed temporal information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ParsedTemporal {
    /// ISO 8601 date if specific
    #[serde(default)]
    pub date: Option<String>,
    /// Relative offset (e.g., -1 for "yesterday")
    #[serde(default)]
    pub relative_days: Option<i32>,
    /// Time of day if mentioned
    #[serde(default)]
    pub time_of_day: Option<String>,
}

/// Types of temporal markers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalMarkerType {
    /// Specific date (January 5th)
    Absolute,
    /// Relative reference (yesterday, last week)
    Relative,
    /// Duration (for 2 hours)
    Duration,
    /// Frequency (every day, weekly)
    Frequency,
    /// Time of day (in the morning)
    TimeOfDay,
}

/// Emotional tone detected in segment
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EmotionalTone {
    /// Primary emotion
    pub primary: String,
    /// Valence (-1.0 negative to 1.0 positive)
    pub valence: f64,
    /// Arousal (0.0 calm to 1.0 excited)
    pub arousal: f64,
    /// Confidence in detection
    pub confidence: f64,
}

// ============================================================================
// Segmentation Output
// ============================================================================

/// Output from segmentation tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentationOutput {
    /// The resulting segments
    pub segments: Vec<Segment>,
    /// Overall segmentation metadata
    pub metadata: SegmentationMetadata,
}

/// Metadata about the segmentation
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SegmentationMetadata {
    /// Input modality
    pub modality: Modality,
    /// Total segments produced
    pub segment_count: usize,
    /// Total entities mentioned
    pub entity_count: usize,
    /// Total temporal markers found
    pub temporal_count: usize,
    /// Primary language detected
    #[serde(default)]
    pub language: Option<String>,
    /// Number of unique speakers (for audio/video)
    #[serde(default)]
    pub speaker_count: Option<usize>,
}

// ============================================================================
// Modality-Specific Features
// ============================================================================

/// Audio-specific segment features
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AudioSegmentFeatures {
    /// Transcribed text
    pub transcript: String,
    /// Speaker ID
    #[serde(default)]
    pub speaker_id: Option<String>,
    /// Audio characteristics
    pub audio_features: AudioFeatures,
}

/// Audio characteristics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AudioFeatures {
    /// Average volume (0.0 to 1.0)
    pub volume: f64,
    /// Speaking rate (words per minute)
    #[serde(default)]
    pub speaking_rate: Option<f64>,
    /// Pitch variation
    #[serde(default)]
    pub pitch_variance: Option<f64>,
    /// Detected emotion from voice
    #[serde(default)]
    pub voice_emotion: Option<String>,
}

/// Vision-specific segment features
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct VisionSegmentFeatures {
    /// Scene description
    pub scene_description: String,
    /// Objects detected
    pub objects: Vec<DetectedObject>,
    /// People detected
    #[serde(default)]
    pub people: Vec<DetectedPerson>,
    /// Actions/activities
    #[serde(default)]
    pub actions: Vec<DetectedAction>,
    /// Text in image (OCR)
    #[serde(default)]
    pub text_content: Option<String>,
}

/// A detected object in vision
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f64,
    #[serde(default)]
    pub bounding_box: Option<BoundingBox>,
}

/// Bounding box for object detection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A detected person in vision
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedPerson {
    #[serde(default)]
    pub face_id: Option<String>,
    #[serde(default)]
    pub emotion: Option<String>,
    #[serde(default)]
    pub bounding_box: Option<BoundingBox>,
}

/// A detected action in vision
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DetectedAction {
    pub action: String,
    pub confidence: f64,
    #[serde(default)]
    pub actor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_serialization() {
        let segment = Segment {
            id: "seg-1".to_string(),
            index: 0,
            content: "I went to the park today".to_string(),
            modality: Modality::Text,
            boundaries: SegmentBoundaries {
                start: 0,
                end: 24,
                speaker_id: None,
            },
            features: SegmentFeatures {
                subject: Some("user".to_string()),
                mentions: vec![],
                temporal_markers: vec![TemporalMarker {
                    text: "today".to_string(),
                    parsed: Some(ParsedTemporal {
                        date: None,
                        relative_days: Some(0),
                        time_of_day: None,
                    }),
                    marker_type: TemporalMarkerType::Relative,
                }],
                emotional_tone: None,
                keywords: vec!["park".to_string()],
                language: Some("en".to_string()),
            },
            confidence: 0.95,
        };

        let json = serde_json::to_string(&segment).unwrap();
        assert!(json.contains("park"));
    }
}
