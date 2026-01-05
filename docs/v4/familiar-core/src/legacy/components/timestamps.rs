//! Timestamps Component
//!
//! Reusable component for entity lifecycle timestamps.
//! Used by entities that track creation and modification times.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Common timestamps for all persistent entities
/// 
/// This component captures:
/// - `created_at`: When the entity was first created
/// - `updated_at`: When the entity was last modified
/// 
/// Use `#[serde(flatten)]` to embed directly into entity structs:
/// ```rust,ignore
/// pub struct MyEntity {
///     pub id: MyId,
///     #[serde(flatten)]
///     pub timestamps: Timestamps,
///     // ... other fields
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Timestamps {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Timestamps {
    fn default() -> Self {
        Self::now()
    }
}

impl Timestamps {
    /// Create new timestamps with current time for both fields
    pub fn now() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }

    /// Create timestamps from specific values (for database reads)
    pub fn from_db(created_at: DateTime<Utc>, updated_at: DateTime<Utc>) -> Self {
        Self { created_at, updated_at }
    }

    /// Update the `updated_at` timestamp to current time
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Get the age since creation
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Get the time since last update
    pub fn since_update(&self) -> chrono::Duration {
        Utc::now() - self.updated_at
    }

    /// Check if entity was modified after creation
    pub fn was_modified(&self) -> bool {
        self.updated_at > self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timestamps_now() {
        let ts = Timestamps::now();
        assert!(ts.created_at <= Utc::now());
        assert!(ts.updated_at <= Utc::now());
        assert_eq!(ts.created_at, ts.updated_at);
    }

    #[test]
    fn test_timestamps_touch() {
        let mut ts = Timestamps::now();
        let original_updated = ts.updated_at;
        
        // Small delay to ensure time difference
        sleep(Duration::from_millis(10));
        ts.touch();
        
        assert!(ts.updated_at > original_updated);
        assert!(ts.was_modified());
    }

    #[test]
    fn test_timestamps_default() {
        let ts = Timestamps::default();
        assert!(!ts.was_modified());
    }
}




