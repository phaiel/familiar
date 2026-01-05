//! Metadata Component
//!
//! Type-safe wrapper for JSON metadata fields.
//! Provides a consistent interface for extensible data across entities.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

/// Type-safe JSON metadata wrapper
/// 
/// Many entities have a `metadata` field for extensible data.
/// This component provides a type-safe interface instead of raw `serde_json::Value`.
///
/// ## Usage
///
/// ```rust,ignore
/// pub struct Message {
///     pub content: String,
///     pub metadata: Metadata,  // Instead of serde_json::Value
/// }
///
/// // Set typed values
/// let mut meta = Metadata::new();
/// meta.set("priority", "high");
/// meta.set("tags", vec!["important", "urgent"]);
///
/// // Get typed values
/// let priority: Option<String> = meta.get("priority");
/// ```
///
/// ## Replaces
/// 
/// This component replaces 14+ occurrences of:
/// - `pub metadata: serde_json::Value`
/// - `pub metadata: Option<serde_json::Value>`
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Metadata(serde_json::Value);

impl Metadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self(serde_json::json!({}))
    }

    /// Create metadata from a JSON value
    pub fn from_value(value: serde_json::Value) -> Self {
        if value.is_object() {
            Self(value)
        } else {
            Self::new()
        }
    }

    /// Get a typed value by key
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.0.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a raw JSON value by key
    pub fn get_raw(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    /// Set a value by key
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        if let Some(obj) = self.0.as_object_mut() {
            if let Ok(json_value) = serde_json::to_value(value) {
                obj.insert(key.to_string(), json_value);
            }
        }
    }

    /// Remove a key
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.0.as_object_mut()
            .and_then(|obj| obj.remove(key))
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.0.get(key).is_some()
    }

    /// Check if metadata is empty
    pub fn is_empty(&self) -> bool {
        self.0.as_object()
            .map(|obj| obj.is_empty())
            .unwrap_or(true)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&str> {
        self.0.as_object()
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get the inner JSON value
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    /// Convert to inner JSON value
    pub fn into_value(self) -> serde_json::Value {
        self.0
    }

    /// Merge another metadata object (other values override self)
    pub fn merge(&mut self, other: &Metadata) {
        if let (Some(base), Some(other_obj)) = (self.0.as_object_mut(), other.0.as_object()) {
            for (key, value) in other_obj {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<serde_json::Value> for Metadata {
    fn from(value: serde_json::Value) -> Self {
        Self::from_value(value)
    }
}

impl From<Metadata> for serde_json::Value {
    fn from(metadata: Metadata) -> Self {
        metadata.0
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Metadata {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_metadata_new() {
        let meta = Metadata::new();
        assert!(meta.is_empty());
    }

    #[test]
    fn test_metadata_set_get() {
        let mut meta = Metadata::new();
        meta.set("key", "value");
        
        let value: Option<String> = meta.get("key");
        assert_eq!(value, Some("value".to_string()));
    }

    #[test]
    fn test_metadata_complex() {
        let mut meta = Metadata::new();
        meta.set("tags", vec!["a", "b", "c"]);
        
        let tags: Option<Vec<String>> = meta.get("tags");
        assert_eq!(tags, Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
    }

    #[test]
    fn test_metadata_merge() {
        let mut base = Metadata::from_value(json!({"a": 1, "b": 2}));
        let other = Metadata::from_value(json!({"b": 3, "c": 4}));
        
        base.merge(&other);
        
        assert_eq!(base.get::<i32>("a"), Some(1));
        assert_eq!(base.get::<i32>("b"), Some(3)); // overwritten
        assert_eq!(base.get::<i32>("c"), Some(4)); // added
    }
}

