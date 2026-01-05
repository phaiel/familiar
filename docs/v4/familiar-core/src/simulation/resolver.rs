//! Thread Resolver - Prevents Ghost Thread Explosion
//!
//! When spawning Bond entities, we need to resolve thread references
//! to existing threads rather than creating new UUIDs each time.
//!
//! This module provides resolution strategies:
//! 1. Exact name match (fast, database query)
//! 2. Fuzzy name match (moderate, Levenshtein distance)
//! 3. Semantic match (slow, requires vector DB like Qdrant)
//!
//! ## Problem Solved
//!
//! Without resolution, saying "I talked to John" ten times creates
//! ten different "John" threads. With resolution, all references
//! to "John" resolve to the same thread entity.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::simulation::resolver::ThreadResolver;
//!
//! let resolver = ThreadResolver::new(db.clone());
//!
//! // Resolve "John" to existing thread or create new
//! let thread_id = resolver.resolve(tenant_id, "John").await?;
//! ```

use crate::primitives::UUID;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, RelationTrait};
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during thread resolution
#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Resolution cache error: {0}")]
    Cache(String),
}

/// Result type for resolver operations
pub type ResolverResult<T> = Result<T, ResolverError>;

/// Configuration for thread resolution
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Enable fuzzy matching (slower but catches typos)
    pub enable_fuzzy: bool,
    /// Maximum Levenshtein distance for fuzzy match
    pub fuzzy_max_distance: usize,
    /// Enable semantic search (requires vector DB)
    pub enable_semantic: bool,
    /// Minimum similarity score for semantic match (0.0 - 1.0)
    pub semantic_min_score: f64,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            enable_fuzzy: true,
            fuzzy_max_distance: 2,
            enable_semantic: false, // Disabled by default (requires Qdrant)
            semantic_min_score: 0.85,
        }
    }
}

/// Thread resolver for preventing duplicate entity creation
///
/// Resolves subject strings to existing thread IDs when possible,
/// only creating new IDs when no match is found.
pub struct ThreadResolver {
    /// Database connection for querying threads
    db: Arc<DatabaseConnection>,
    /// Resolution configuration
    config: ResolverConfig,
    /// In-memory cache for recent resolutions
    cache: std::sync::RwLock<std::collections::HashMap<(uuid::Uuid, String), UUID>>,
}

impl ThreadResolver {
    /// Create a new thread resolver with database connection
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db,
            config: ResolverConfig::default(),
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
    
    /// Create a new thread resolver with custom configuration
    pub fn with_config(db: Arc<DatabaseConnection>, config: ResolverConfig) -> Self {
        Self {
            db,
            config,
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
    
    /// Resolve a subject string to a thread ID
    ///
    /// Resolution order:
    /// 1. Check cache
    /// 2. Exact match on thread name
    /// 3. Fuzzy match (if enabled)
    /// 4. Semantic match (if enabled and Qdrant available)
    /// 5. Create new UUID if no match
    pub async fn resolve(&self, tenant_id: UUID, subject: &str) -> ResolverResult<UUID> {
        let tenant_uuid = tenant_id.as_uuid();
        let normalized = normalize_subject(subject);
        
        // 1. Check cache
        if let Some(cached) = self.get_cached(tenant_uuid, &normalized) {
            return Ok(cached);
        }
        
        // 2. Exact match
        if let Some(id) = self.find_exact_match(tenant_uuid, &normalized).await? {
            self.cache_resolution(tenant_uuid, &normalized, id);
            return Ok(id);
        }
        
        // 3. Fuzzy match (if enabled)
        if self.config.enable_fuzzy {
            if let Some(id) = self.find_fuzzy_match(tenant_uuid, &normalized).await? {
                self.cache_resolution(tenant_uuid, &normalized, id);
                return Ok(id);
            }
        }
        
        // 4. Semantic match would go here (requires Qdrant integration)
        // if self.config.enable_semantic {
        //     if let Some(id) = self.find_semantic_match(tenant_uuid, &normalized).await? {
        //         return Ok(id);
        //     }
        // }
        
        // 5. No match found - create new UUID
        let new_id = UUID::new();
        self.cache_resolution(tenant_uuid, &normalized, new_id);
        Ok(new_id)
    }
    
    /// Find exact match in database
    /// 
    /// Joins entity_registry with comp_content to match on text_content
    async fn find_exact_match(&self, tenant_id: uuid::Uuid, subject: &str) -> ResolverResult<Option<UUID>> {
        use crate::entities::db::physics::{entity_registry, content};
        use sea_orm::JoinType;
        use sea_orm::QuerySelect;
        
        // Query entity_registry for threads, joining with content to match text
        let result = entity_registry::Entity::find()
            .join(JoinType::InnerJoin, entity_registry::Relation::Content.def())
            .filter(entity_registry::Column::TenantId.eq(tenant_id))
            .filter(entity_registry::Column::EntityType.eq("Thread"))
            .filter(content::Column::TextContent.eq(subject))
            .one(self.db.as_ref())
            .await?;
        
        Ok(result.map(|r| UUID::from_uuid(r.id.as_uuid())))
    }
    
    /// Find fuzzy match using Levenshtein distance
    /// 
    /// Loads all threads for a tenant and compares text content
    async fn find_fuzzy_match(&self, tenant_id: uuid::Uuid, subject: &str) -> ResolverResult<Option<UUID>> {
        use crate::entities::db::physics::{entity_registry, content};
        use sea_orm::JoinType;
        use sea_orm::QuerySelect;
        
        // Get all threads for this tenant with their content
        let threads = entity_registry::Entity::find()
            .join(JoinType::InnerJoin, entity_registry::Relation::Content.def())
            .filter(entity_registry::Column::TenantId.eq(tenant_id))
            .filter(entity_registry::Column::EntityType.eq("Thread"))
            .find_also_related(content::Entity)
            .all(self.db.as_ref())
            .await?;
        
        // Find best fuzzy match based on content text
        let best_match = threads.iter()
            .filter_map(|(entity, maybe_content)| {
                if let Some(content) = maybe_content {
                    let text = &content.text_content;
                    let distance = levenshtein_distance(subject, text);
                    if distance <= self.config.fuzzy_max_distance {
                        Some((entity, distance))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .min_by_key(|(_, dist)| *dist);
        
        Ok(best_match.map(|(entity, _)| UUID::from_uuid(entity.id.as_uuid())))
    }
    
    /// Get cached resolution
    fn get_cached(&self, tenant_id: uuid::Uuid, subject: &str) -> Option<UUID> {
        let cache = self.cache.read().ok()?;
        cache.get(&(tenant_id, subject.to_string())).copied()
    }
    
    /// Cache a resolution
    fn cache_resolution(&self, tenant_id: uuid::Uuid, subject: &str, id: UUID) {
        if let Ok(mut cache) = self.cache.write() {
            // Limit cache size
            if cache.len() > 10000 {
                cache.clear();
            }
            cache.insert((tenant_id, subject.to_string()), id);
        }
    }
    
    /// Clear the resolution cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

/// Normalize a subject string for matching
fn normalize_subject(subject: &str) -> String {
    subject
        .trim()
        .to_lowercase()
        // Remove common prefixes
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim_start_matches("an ")
        .to_string()
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];
    
    for i in 0..=a_len { matrix[i][0] = i; }
    for j in 0..=b_len { matrix[0][j] = j; }
    
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,     // deletion
                    matrix[i][j - 1] + 1,     // insertion
                ),
                matrix[i - 1][j - 1] + cost,   // substitution
            );
        }
    }
    
    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_subject() {
        assert_eq!(normalize_subject("  John  "), "john");
        assert_eq!(normalize_subject("The Meeting"), "meeting");
        assert_eq!(normalize_subject("A Project"), "project");
    }
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("john", "jon"), 1);
        assert_eq!(levenshtein_distance("john", "john"), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }
}

