//! Tower Service Layers
//!
//! Provides reusable middleware for:
//! - Rate limiting (protect LLM APIs)
//! - Batching (compact DB writes)
//! - Timeout handling
//! - Metrics collection
//!
//! Configuration is via environment variables (Cloud-Native pattern):
//! - `BATCH_SIZE`: Max items per batch (default: 200)
//! - `BATCH_LINGER_MS`: Max wait time for batch (default: 100)
//! - `RATE_LIMIT_RPS`: Requests per second (default: 100)
//! - `TIMEOUT_SECS`: Operation timeout (default: 30)

use std::time::Duration;
use tracing::info;

/// Configuration for Tower layers
#[derive(Debug, Clone)]
pub struct LayerConfig {
    /// Max items per batch
    pub batch_size: usize,
    /// Max wait time for batch in milliseconds
    pub batch_linger_ms: u64,
    /// Requests per second limit
    pub rate_limit_rps: u64,
    /// Operation timeout in seconds
    pub timeout_secs: u64,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            batch_size: std::env::var("BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(200),
            batch_linger_ms: std::env::var("BATCH_LINGER_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            rate_limit_rps: std::env::var("RATE_LIMIT_RPS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            timeout_secs: std::env::var("TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}

impl LayerConfig {
    /// Create config for high-throughput scenarios (e.g., Fates pipeline)
    pub fn high_throughput() -> Self {
        Self {
            batch_size: 500,
            batch_linger_ms: 50,
            rate_limit_rps: 2000,
            timeout_secs: 60,
        }
    }

    /// Create config for LLM-heavy scenarios (rate-limited)
    pub fn llm_heavy() -> Self {
        Self {
            batch_size: 10,
            batch_linger_ms: 200,
            rate_limit_rps: 50,
            timeout_secs: 120,
        }
    }

    /// Create config for database-heavy scenarios
    pub fn database_heavy() -> Self {
        Self {
            batch_size: 200,
            batch_linger_ms: 100,
            rate_limit_rps: 500,
            timeout_secs: 30,
        }
    }

    /// Get batch linger duration
    pub fn batch_linger(&self) -> Duration {
        Duration::from_millis(self.batch_linger_ms)
    }

    /// Get timeout duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Log the configuration
    pub fn log(&self) {
        info!(
            batch_size = self.batch_size,
            batch_linger_ms = self.batch_linger_ms,
            rate_limit_rps = self.rate_limit_rps,
            timeout_secs = self.timeout_secs,
            "Tower layer configuration"
        );
    }
}

/// Record a request metric
///
/// Uses the metrics crate 0.21 API.
/// In 0.21, the macros directly increment/record; they don't return a handle.
pub fn record_request(domain: &str, action: &str, success: bool, duration_secs: f64) {
    let domain = domain.to_string();
    let action = action.to_string();
    
    if success {
        metrics::increment_counter!(
            "minerva_requests_success_total",
            "domain" => domain.clone(),
            "action" => action.clone()
        );
    } else {
        metrics::increment_counter!(
            "minerva_requests_failed_total",
            "domain" => domain.clone(),
            "action" => action.clone()
        );
    }

    metrics::histogram!(
        "minerva_request_duration_seconds", 
        duration_secs,
        "domain" => domain,
        "action" => action
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LayerConfig::default();
        assert_eq!(config.batch_size, 200);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_high_throughput_config() {
        let config = LayerConfig::high_throughput();
        assert_eq!(config.rate_limit_rps, 2000);
    }

    #[test]
    fn test_durations() {
        let config = LayerConfig::default();
        assert_eq!(config.batch_linger(), Duration::from_millis(100));
        assert_eq!(config.timeout(), Duration::from_secs(30));
    }
}
