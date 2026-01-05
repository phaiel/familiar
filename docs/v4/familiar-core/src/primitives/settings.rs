//! Settings Primitive
//!
//! Type-safe wrapper for JSON settings with merge and query support.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

/// Type-safe settings wrapper with merge support
/// 
/// Settings are stored as JSON but provide a type-safe interface:
/// - Get typed values by key
/// - Set values by key
/// - Merge with other settings (deep merge)
/// - Default to empty object
///
/// ## Usage
///
/// ```rust,ignore
/// let mut settings = Settings::default();
/// 
/// // Set values
/// settings.set("theme", "dark");
/// settings.set("notifications", json!({"email": true, "push": false}));
///
/// // Get typed values
/// let theme: Option<String> = settings.get("theme");
/// let notifications: Option<NotificationConfig> = settings.get("notifications");
///
/// // Merge with another settings object
/// let other = Settings::from_value(json!({"theme": "light"}));
/// settings.merge(&other); // theme is now "light"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Settings(serde_json::Value);

impl Settings {
    /// Create new empty settings
    pub fn new() -> Self {
        Self(serde_json::json!({}))
    }

    /// Create settings from a JSON value
    pub fn from_value(value: serde_json::Value) -> Self {
        if value.is_object() {
            Self(value)
        } else {
            Self::new()
        }
    }

    /// Get a typed value by key
    /// 
    /// Returns `None` if key doesn't exist or can't be deserialized to type T
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

    /// Get all keys
    pub fn keys(&self) -> Vec<&str> {
        self.0.as_object()
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if settings is empty
    pub fn is_empty(&self) -> bool {
        self.0.as_object()
            .map(|obj| obj.is_empty())
            .unwrap_or(true)
    }

    /// Get the inner JSON value
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    /// Convert to inner JSON value
    pub fn into_value(self) -> serde_json::Value {
        self.0
    }

    /// Merge another settings object into this one
    /// 
    /// This performs a deep merge:
    /// - Scalar values from `other` override values in `self`
    /// - Object values are recursively merged
    /// - Array values from `other` replace arrays in `self`
    pub fn merge(&mut self, other: &Settings) {
        Self::deep_merge(&mut self.0, &other.0);
    }

    /// Deep merge two JSON values
    fn deep_merge(base: &mut serde_json::Value, other: &serde_json::Value) {
        match (base, other) {
            (serde_json::Value::Object(base_obj), serde_json::Value::Object(other_obj)) => {
                for (key, value) in other_obj {
                    if let Some(base_value) = base_obj.get_mut(key) {
                        Self::deep_merge(base_value, value);
                    } else {
                        base_obj.insert(key.clone(), value.clone());
                    }
                }
            }
            (base, other) => {
                *base = other.clone();
            }
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<serde_json::Value> for Settings {
    fn from(value: serde_json::Value) -> Self {
        Self::from_value(value)
    }
}

impl From<Settings> for serde_json::Value {
    fn from(settings: Settings) -> Self {
        settings.0
    }
}

impl PartialEq for Settings {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Settings {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_settings_new() {
        let settings = Settings::new();
        assert!(settings.is_empty());
    }

    #[test]
    fn test_settings_set_get() {
        let mut settings = Settings::new();
        settings.set("theme", "dark");
        
        let theme: Option<String> = settings.get("theme");
        assert_eq!(theme, Some("dark".to_string()));
    }

    #[test]
    fn test_settings_complex_value() {
        let mut settings = Settings::new();
        settings.set("config", json!({"nested": {"value": 42}}));
        
        let raw = settings.get_raw("config");
        assert!(raw.is_some());
        assert_eq!(raw.unwrap()["nested"]["value"], 42);
    }

    #[test]
    fn test_settings_merge() {
        let mut base = Settings::from_value(json!({
            "a": 1,
            "b": {"c": 2, "d": 3}
        }));
        
        let other = Settings::from_value(json!({
            "a": 10,
            "b": {"c": 20, "e": 5}
        }));
        
        base.merge(&other);
        
        // a should be overwritten
        assert_eq!(base.get::<i32>("a"), Some(10));
        
        // b.c should be overwritten, b.d preserved, b.e added
        let b = base.get_raw("b").unwrap();
        assert_eq!(b["c"], 20);
        assert_eq!(b["d"], 3);
        assert_eq!(b["e"], 5);
    }

    #[test]
    fn test_settings_remove() {
        let mut settings = Settings::from_value(json!({"a": 1, "b": 2}));
        
        let removed = settings.remove("a");
        assert_eq!(removed, Some(json!(1)));
        assert!(!settings.contains("a"));
        assert!(settings.contains("b"));
    }

    #[test]
    fn test_settings_keys() {
        let settings = Settings::from_value(json!({"a": 1, "b": 2}));
        let mut keys = settings.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }
}

