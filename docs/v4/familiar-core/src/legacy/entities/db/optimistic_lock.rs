//! Optimistic Locking Utilities
//!
//! Provides helpers for implementing optimistic locking in SeaORM entities.
//! This prevents lost updates when multiple actors (API, Windmill, Worker)
//! access the same records.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use familiar_core::entities::db::optimistic_lock::{OptimisticLockError, with_optimistic_lock};
//! use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
//!
//! // 1. Read the record
//! let task = AsyncTask::find_by_id(id).one(&db).await?;
//! let original_version = task.version;
//!
//! // 2. Update with version check
//! let result = with_optimistic_lock(
//!     &db,
//!     async_task::Entity,
//!     async_task::Column::Id,
//!     id,
//!     async_task::Column::Version,
//!     original_version,
//!     |mut model| {
//!         model.status = Set(TaskStatus::Running);
//!         model.version = Set(original_version + 1);
//!         model
//!     },
//! ).await?;
//! ```

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter,
};
use thiserror::Error;

/// Error type for optimistic locking failures
#[derive(Error, Debug)]
pub enum OptimisticLockError {
    /// The record was modified by another actor (version mismatch)
    #[error("Optimistic lock failed: record was modified by another actor (expected version {expected}, current version has changed)")]
    VersionMismatch {
        expected: i32,
    },
    
    /// The record was not found
    #[error("Record not found")]
    NotFound,
    
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Result type for optimistic locking operations
pub type OptimisticLockResult<T> = Result<T, OptimisticLockError>;

/// Check if an update affected rows (for version checking)
/// 
/// Returns `Ok(())` if exactly one row was affected, otherwise returns an error.
pub fn check_update_result(
    update_result: sea_orm::UpdateResult,
    expected_version: i32,
) -> OptimisticLockResult<()> {
    if update_result.rows_affected == 0 {
        Err(OptimisticLockError::VersionMismatch {
            expected: expected_version,
        })
    } else {
        Ok(())
    }
}

/// Trait for entities that support optimistic locking
pub trait OptimisticLock {
    /// Get the current version
    fn version(&self) -> i32;
}

/// Macro to implement OptimisticLock for SeaORM entities
/// 
/// Usage:
/// ```rust,ignore
/// impl_optimistic_lock!(async_task::Model);
/// ```
#[macro_export]
macro_rules! impl_optimistic_lock {
    ($model:ty) => {
        impl $crate::entities::db::optimistic_lock::OptimisticLock for $model {
            fn version(&self) -> i32 {
                self.version
            }
        }
    };
}

// Implement OptimisticLock for our entities
impl OptimisticLock for super::task::async_task::Model {
    fn version(&self) -> i32 {
        self.version
    }
}

impl OptimisticLock for super::auth::session::Model {
    fn version(&self) -> i32 {
        self.version
    }
}

impl OptimisticLock for super::conversation::channel::Model {
    fn version(&self) -> i32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_update_result_success() {
        let result = sea_orm::UpdateResult { rows_affected: 1 };
        assert!(check_update_result(result, 0).is_ok());
    }

    #[test]
    fn test_check_update_result_version_mismatch() {
        let result = sea_orm::UpdateResult { rows_affected: 0 };
        let err = check_update_result(result, 5).unwrap_err();
        match err {
            OptimisticLockError::VersionMismatch { expected } => {
                assert_eq!(expected, 5);
            }
            _ => panic!("Expected VersionMismatch error"),
        }
    }
}






