//! API Types
//!
//! Standard types for API request/response handling:
//! - `ApiResult<T>`: Standard success/error wrapper
//! - `ApiError`: Structured error details
//! - `ListResult<T>`: Paginated list response
//! - `SuccessResult`: Simple success indicator

pub mod responses;

pub use self::responses::{ApiResult, ApiError, ListResult, SuccessResult};




