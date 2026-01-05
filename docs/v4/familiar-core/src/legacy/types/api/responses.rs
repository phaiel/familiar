//! API Response Wrappers
//!
//! Standard response types for API endpoints.
//! These provide consistent structure for success, error, and list responses.

use serde::{Deserialize, Serialize};

/// Standard API success/error wrapper
/// 
/// Wraps any response type `T` with standard metadata:
/// - `success`: Whether the operation succeeded
/// - `data`: The actual response data (when successful)
/// - `error`: Error details (when failed)
///
/// ## Usage
///
/// ```rust,ignore
/// // Success case
/// let response = ApiResult::ok(my_data);
///
/// // Error case  
/// let response = ApiResult::<MyData>::err("Something went wrong", "OPERATION_FAILED");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ApiResult<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl<T> ApiResult<T> {
    /// Create a successful response with data
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response
    pub fn err(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError::new(message, code)),
        }
    }

    /// Create an error response with details
    pub fn err_with_details(
        message: impl Into<String>, 
        code: impl Into<String>,
        details: serde_json::Value
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError::with_details(message, code, details)),
        }
    }

    /// Check if this is a success response
    pub fn is_ok(&self) -> bool {
        self.success
    }

    /// Check if this is an error response
    pub fn is_err(&self) -> bool {
        !self.success
    }

    /// Get the data, panics if error
    pub fn unwrap(self) -> T {
        self.data.expect("Called unwrap on an error ApiResult")
    }

    /// Get the data, returns None if error
    pub fn into_option(self) -> Option<T> {
        self.data
    }
}

impl<T: Default> Default for ApiResult<T> {
    fn default() -> Self {
        Self::ok(T::default())
    }
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ApiError {
    /// Error code (e.g., "NOT_FOUND", "VALIDATION_FAILED")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional structured details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create an API error with details
    pub fn with_details(
        message: impl Into<String>, 
        code: impl Into<String>,
        details: serde_json::Value
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Common error codes
    pub const NOT_FOUND: &'static str = "NOT_FOUND";
    pub const UNAUTHORIZED: &'static str = "UNAUTHORIZED";
    pub const FORBIDDEN: &'static str = "FORBIDDEN";
    pub const VALIDATION_FAILED: &'static str = "VALIDATION_FAILED";
    pub const INTERNAL_ERROR: &'static str = "INTERNAL_ERROR";
    pub const CONFLICT: &'static str = "CONFLICT";
    pub const RATE_LIMITED: &'static str = "RATE_LIMITED";
}

/// Paginated list response
/// 
/// Used for endpoints that return multiple items with pagination:
/// - `items`: The list of items
/// - `count`: Total number of items in this response
/// - `total`: Total number of items available (optional)
/// - `cursor`: Cursor for fetching next page (optional)
///
/// ## Usage
///
/// ```rust,ignore
/// let response = ListResult::new(items);
/// let response = ListResult::new(items).with_cursor("next_cursor");
/// let response = ListResult::new(items).with_total(1000);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl<T> ListResult<T> {
    /// Create a new list result
    pub fn new(items: Vec<T>) -> Self {
        let count = items.len();
        Self {
            items,
            count,
            total: None,
            cursor: None,
        }
    }

    /// Create an empty list result
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            count: 0,
            total: Some(0),
            cursor: None,
        }
    }

    /// Add a cursor for pagination
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Add total count for pagination
    pub fn with_total(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }

    /// Check if there are more items
    pub fn has_more(&self) -> bool {
        self.cursor.is_some()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Default for ListResult<T> {
    fn default() -> Self {
        Self::empty()
    }
}

/// Simple success response (no data)
/// 
/// Used for operations that succeed but don't return data:
/// - DELETE operations
/// - Fire-and-forget operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SuccessResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl SuccessResult {
    /// Create a success result
    pub fn ok() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    /// Create a success result with a message
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
        }
    }
}

impl Default for SuccessResult {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_result_ok() {
        let result: ApiResult<String> = ApiResult::ok("hello".to_string());
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_api_result_err() {
        let result: ApiResult<String> = ApiResult::err("Something failed", "FAILURE");
        assert!(result.is_err());
        assert!(!result.is_ok());
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap().code, "FAILURE");
    }

    #[test]
    fn test_list_result() {
        let items = vec!["a", "b", "c"];
        let result = ListResult::new(items).with_total(100).with_cursor("next");
        
        assert_eq!(result.count, 3);
        assert_eq!(result.total, Some(100));
        assert!(result.has_more());
    }

    #[test]
    fn test_list_result_empty() {
        let result: ListResult<String> = ListResult::empty();
        assert!(result.is_empty());
        assert!(!result.has_more());
    }
}




