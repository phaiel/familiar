//! Nutype-based Primitive Macros
//!
//! These macros reduce boilerplate when defining primitive wrapper types
//! while maintaining compatibility with serde and sqlx.
//!
//! # Example
//!
//! ```rust,ignore
//! define_uuid_id!(UserId, "A user's unique identifier");
//! define_string_primitive!(Email, "An email address", |s| s.contains('@'));
//! ```

/// Define a UUID-based ID primitive type with standard derives.
///
/// Generates:
/// - nutype wrapper with Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize
/// - Display impl (shows UUID)
/// - From<Uuid> and Into<Uuid> conversions
/// - Default impl (generates new random UUID)
/// - sqlx::Type for database compatibility
///
/// # Example
///
/// ```rust,ignore
/// define_uuid_id!(UserId, "A user's unique identifier");
/// 
/// let id = UserId::new();
/// println!("User: {}", id);
/// ```
#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[derive(sqlx::Type)]
        #[serde(transparent)]
        #[sqlx(transparent)]
        pub struct $name(pub(crate) uuid::Uuid);

        impl $name {
            /// Create a new random ID
            #[inline]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            /// Parse from a string
            #[inline]
            pub fn parse(s: &str) -> Result<Self, uuid::Error> {
                Ok(Self(uuid::Uuid::parse_str(s)?))
            }

            /// Get the inner UUID (for database operations)
            #[inline]
            pub fn as_uuid(&self) -> uuid::Uuid {
                self.0
            }

            /// Create from a UUID (for database results)
            #[inline]
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for uuid::Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl AsRef<uuid::Uuid> for $name {
            fn as_ref(&self) -> &uuid::Uuid {
                &self.0
            }
        }
    };
}

/// Define a validated string primitive type.
///
/// Generates:
/// - String wrapper with Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize
/// - Validation via a provided closure
/// - Display impl
/// - TryFrom<String> with validation
///
/// # Example
///
/// ```rust,ignore
/// define_validated_string!(Email, "An email address", |s: &str| s.contains('@'));
/// 
/// let email = Email::new("user@example.com")?;
/// ```
#[macro_export]
macro_rules! define_validated_string {
    ($name:ident, $doc:literal, $validate:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Create a new instance with validation
            pub fn new(value: impl Into<String>) -> Result<Self, String> {
                let s: String = value.into();
                let validator: fn(&str) -> bool = $validate;
                if validator(&s) {
                    Ok(Self(s))
                } else {
                    Err(format!("Invalid {}: '{}'", stringify!($name), s))
                }
            }

            /// Create without validation (unsafe - use only when you know value is valid)
            pub fn new_unchecked(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Get the inner string
            #[inline]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume and return the inner string
            #[inline]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = String;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = String;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

/// Define a simple string wrapper (no validation).
///
/// Use this for strings that don't need runtime validation but
/// benefit from type safety.
///
/// # Example
///
/// ```rust,ignore
/// define_string_wrapper!(PasswordHash, "A hashed password");
/// ```
#[macro_export]
macro_rules! define_string_wrapper {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Create a new instance
            #[inline]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Get the inner string
            #[inline]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume and return the inner string
            #[inline]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<$name> for String {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    define_uuid_id!(TestUserId, "Test user ID");
    define_validated_string!(TestEmail, "Test email", |s: &str| s.contains('@'));
    define_string_wrapper!(TestPassword, "Test password");

    #[test]
    fn test_uuid_id() {
        let id = TestUserId::new();
        assert!(!id.to_string().is_empty());

        let parsed = TestUserId::parse(&id.to_string()).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_validated_string() {
        let email = TestEmail::new("test@example.com").unwrap();
        assert_eq!(email.as_str(), "test@example.com");

        let invalid = TestEmail::new("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_string_wrapper() {
        let password = TestPassword::new("secret123");
        assert_eq!(password.as_str(), "secret123");
    }
}




