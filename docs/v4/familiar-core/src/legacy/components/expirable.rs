//! Expirable Component
//!
//! Trait for types that have an expiration time.
//! Provides consistent expiration checking across entities.

use chrono::{DateTime, Duration, Utc};

/// Trait for types with expiration
///
/// Implement this for any type that has a time-based validity window.
/// This is commonly used for sessions, tokens, invitations, and temporary records.
///
/// ## Types That Implement This
///
/// - `AuthSession` - User sessions with configurable lifetime
/// - `MagicLink` - One-time login links
/// - `FamilyInvitation` - Time-limited invitations
/// - `AsyncTask` - Tasks with timeout deadlines
pub trait Expirable {
    /// Get the expiration time, if any
    fn expires_at(&self) -> Option<DateTime<Utc>>;

    /// Check if the item has expired
    fn is_expired(&self) -> bool {
        self.expires_at()
            .map(|exp| exp <= Utc::now())
            .unwrap_or(false) // No expiration = never expires
    }

    /// Check if the item is still valid
    fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Get time remaining until expiration
    fn time_remaining(&self) -> Option<Duration> {
        self.expires_at().map(|exp| {
            let now = Utc::now();
            if exp > now {
                exp - now
            } else {
                Duration::zero()
            }
        })
    }

    /// Check if expiring soon (within given duration)
    fn expires_soon(&self, within: Duration) -> bool {
        self.time_remaining()
            .map(|remaining| remaining < within)
            .unwrap_or(false)
    }

    /// Check if expiring within the next hour
    fn expires_within_hour(&self) -> bool {
        self.expires_soon(Duration::hours(1))
    }

    /// Get a human-readable status string
    fn expiration_status(&self) -> &'static str {
        match self.expires_at() {
            None => "Never expires",
            Some(exp) => {
                let now = Utc::now();
                if exp <= now {
                    "Expired"
                } else if exp - now < Duration::hours(1) {
                    "Expires soon"
                } else if exp - now < Duration::days(1) {
                    "Expires today"
                } else {
                    "Valid"
                }
            }
        }
    }
}

/// Helper to check expiration of optional datetime
pub fn is_expired(expires_at: Option<DateTime<Utc>>) -> bool {
    expires_at
        .map(|exp| exp <= Utc::now())
        .unwrap_or(false)
}

/// Helper to get time remaining from optional datetime
pub fn time_remaining(expires_at: Option<DateTime<Utc>>) -> Option<Duration> {
    expires_at.map(|exp| {
        let now = Utc::now();
        if exp > now {
            exp - now
        } else {
            Duration::zero()
        }
    })
}

/// Calculate an expiration time from now
pub fn expires_in(duration: Duration) -> DateTime<Utc> {
    Utc::now() + duration
}

/// Calculate an expiration time in hours from now
pub fn expires_in_hours(hours: i64) -> DateTime<Utc> {
    expires_in(Duration::hours(hours))
}

/// Calculate an expiration time in days from now
pub fn expires_in_days(days: i64) -> DateTime<Utc> {
    expires_in(Duration::days(days))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestToken {
        expires_at: Option<DateTime<Utc>>,
    }

    impl Expirable for TestToken {
        fn expires_at(&self) -> Option<DateTime<Utc>> {
            self.expires_at
        }
    }

    #[test]
    fn test_not_expired() {
        let token = TestToken {
            expires_at: Some(Utc::now() + Duration::hours(1)),
        };
        assert!(!token.is_expired());
        assert!(token.is_valid());
    }

    #[test]
    fn test_expired() {
        let token = TestToken {
            expires_at: Some(Utc::now() - Duration::hours(1)),
        };
        assert!(token.is_expired());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_never_expires() {
        let token = TestToken { expires_at: None };
        assert!(!token.is_expired());
        assert!(token.is_valid());
    }

    #[test]
    fn test_time_remaining() {
        let token = TestToken {
            expires_at: Some(Utc::now() + Duration::hours(2)),
        };
        let remaining = token.time_remaining().unwrap();
        assert!(remaining > Duration::minutes(119));
        assert!(remaining <= Duration::hours(2));
    }

    #[test]
    fn test_expires_soon() {
        let token = TestToken {
            expires_at: Some(Utc::now() + Duration::minutes(30)),
        };
        assert!(token.expires_within_hour());
        assert!(token.expires_soon(Duration::hours(1)));
        assert!(!token.expires_soon(Duration::minutes(15)));
    }

    #[test]
    fn test_expiration_helpers() {
        let exp = expires_in_hours(24);
        assert!(exp > Utc::now());
        assert!(time_remaining(Some(exp)).unwrap() > Duration::hours(23));
    }
}




