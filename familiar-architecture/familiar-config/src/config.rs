use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global configuration structure for the Familiar platform.
/// This serves as the central authority for all operational parameters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    /// Node-specific resource allocations and constraints
    pub nodes: NodeResources,
    
    /// System-specific timeouts, retries, and operational parameters
    pub systems: SystemConfig,
    
    /// Observability thresholds and monitoring settings
    pub observability: ObservabilityConfig,
    
    /// Infrastructure-level settings and defaults
    pub infra: InfraConfig,

    /// Profile-Guided Optimization settings
    pub pgo: PgoConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            nodes: NodeResources::default(),
            systems: SystemConfig::default(),
            observability: ObservabilityConfig::default(),
            infra: InfraConfig::default(),
            pgo: PgoConfig::default(),
        }
    }
}

impl GlobalConfig {
    /// Load configuration from multiple sources with layered overrides
    pub fn load() -> anyhow::Result<Self> {
        let mut builder = config::Config::builder();
        
        // Try to load from crate root config directory first
        let crate_config_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
        if crate_config_dir.exists() {
            builder = builder
                .add_source(config::File::from(crate_config_dir.join("defaults")).required(false))
                .add_source(config::File::from(crate_config_dir.join("production")).required(false))
                .add_source(config::File::from(crate_config_dir.join("staging")).required(false))
                .add_source(config::File::from(crate_config_dir.join("development")).required(false));
        }
        
        // Try to load from working directory config
        builder = builder
            .add_source(config::File::with_name("config/defaults").required(false))
            .add_source(config::File::with_name("config/production").required(false))
            .add_source(config::File::with_name("config/staging").required(false))
            .add_source(config::File::with_name("config/development").required(false));
        
        // Load environment variables with FAMILIAR_ prefix
        builder = builder.add_source(config::Environment::with_prefix("FAMILIAR").separator("_"));

        let config = builder.build()?;
        let global_config: GlobalConfig = config.try_deserialize()?;
        
        Ok(global_config)
    }
    
    /// Get a thread-local reference to the loaded configuration
    pub fn get() -> &'static GlobalConfig {
        use std::sync::OnceLock;
        static CONFIG: OnceLock<GlobalConfig> = OnceLock::new();
        
        CONFIG.get_or_init(|| {
            Self::load().expect("Failed to load configuration")
        })
    }
}

// =============================================================================
// Node Resources Configuration
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeResources {
    pub familiar_daemon: NodeResourceConfig,
    pub familiar_worker: NodeResourceConfig,
    pub classifier: NodeResourceConfig,
}

impl Default for NodeResources {
    fn default() -> Self {
        Self {
            familiar_daemon: NodeResourceConfig::default(),
            familiar_worker: NodeResourceConfig::default(),
            classifier: NodeResourceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeResourceConfig {
    /// Resource allocations
    pub resources: NodeResourcesAllocation,
    
    /// Runtime constraints and thresholds
    pub constraints: NodeConstraints,
}

impl Default for NodeResourceConfig {
    fn default() -> Self {
        Self {
            resources: NodeResourcesAllocation::default(),
            constraints: NodeConstraints::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeResourcesAllocation {
    /// CPU allocation in milli-cores (e.g., "2000m")
    pub cpu: String,
    
    /// Memory allocation (e.g., "8Gi")
    pub memory: String,
    
    /// Storage allocation (e.g., "50Gi")
    pub storage: String,
    
    /// GPU count (optional)
    #[serde(default)]
    pub gpu: Option<u32>,
}

impl Default for NodeResourcesAllocation {
    fn default() -> Self {
        Self {
            cpu: "1000m".to_string(),
            memory: "1Gi".to_string(),
            storage: "10Gi".to_string(),
            gpu: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeConstraints {
    /// Memory threshold for health checks (bytes)
    pub memory_threshold: u64,

    /// CPU usage threshold (0.0-1.0)
    pub cpu_threshold: f64,

    /// Schema version requirement
    pub schema_version: String,

    /// Queue depth threshold
    pub queue_depth_threshold: usize,

    /// Error rate threshold (0.0-1.0)
    pub error_rate_threshold: f64,

    /// Active job limits for specialized workloads
    #[serde(default)]
    pub active_job_limits: Option<NodeJobLimits>,
}

impl Default for NodeConstraints {
    fn default() -> Self {
        Self {
            memory_threshold: 1_000_000, // 1MB
            cpu_threshold: 0.8,
            schema_version: "1.0.0".to_string(),
            queue_depth_threshold: 100,
            error_rate_threshold: 0.05,
            active_job_limits: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJobLimits {
    pub llm_jobs: usize,
    pub ml_jobs: usize,
    pub db_connections: usize,
}

// =============================================================================
// System Configuration
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemConfig {
    pub fates_gate: FatesGateConfig,
    pub classifier_system: ClassifierSystemConfig,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            fates_gate: FatesGateConfig::default(),
            classifier_system: ClassifierSystemConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FatesGateConfig {
    /// Timeout configurations for different dispatch routes
    pub timeouts: FatesGateTimeouts,
    
    /// Retry configurations
    pub retries: FatesGateRetries,
    
    /// Routing policy parameters
    pub routing: FatesGateRouting,
}

impl Default for FatesGateConfig {
    fn default() -> Self {
        Self {
            timeouts: FatesGateTimeouts::default(),
            retries: FatesGateRetries::default(),
            routing: FatesGateRouting::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FatesGateTimeouts {
    pub weave: Duration,
    pub search: Duration,
    pub classify: Duration,
}

impl Default for FatesGateTimeouts {
    fn default() -> Self {
        Self {
            weave: Duration::from_secs(30),
            search: Duration::from_secs(60),
            classify: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FatesGateRetries {
    pub weave: u32,
    pub search: u32,
    pub classify: u32,
}

impl Default for FatesGateRetries {
    fn default() -> Self {
        Self {
            weave: 3,
            search: 2,
            classify: 1,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FatesGateRouting {
    pub memory_threshold: f64,
    pub complexity_threshold: f64,
}

impl Default for FatesGateRouting {
    fn default() -> Self {
        Self {
            memory_threshold: 10000.0,
            complexity_threshold: 0.8,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassifierSystemConfig {
    pub timeouts: ClassifierTimeouts,
    pub retries: ClassifierRetries,
    pub routing: ClassifierRouting,
}

impl Default for ClassifierSystemConfig {
    fn default() -> Self {
        Self {
            timeouts: ClassifierTimeouts::default(),
            retries: ClassifierRetries::default(),
            routing: ClassifierRouting::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassifierTimeouts {
    pub classification: Duration,
    pub entity_segment: Duration,
    pub purpose_classification: Duration,
}

impl Default for ClassifierTimeouts {
    fn default() -> Self {
        Self {
            classification: Duration::from_secs(120),
            entity_segment: Duration::from_secs(30),
            purpose_classification: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassifierRetries {
    pub classification: u32,
    pub entity_segment: u32,
    pub purpose_classification: u32,
}

impl Default for ClassifierRetries {
    fn default() -> Self {
        Self {
            classification: 2,
            entity_segment: 3,
            purpose_classification: 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassifierRouting {
    pub urgency_threshold: String,
    pub gpu_pool_threshold: String,
}

impl Default for ClassifierRouting {
    fn default() -> Self {
        Self {
            urgency_threshold: "high".to_string(),
            gpu_pool_threshold: "gpu-pool".to_string(),
        }
    }
}

// =============================================================================
// Observability Configuration
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    pub circuit_breaker: CircuitBreakerConfig,
    pub load_shedding: LoadSheddingConfig,
    pub metrics: MetricsConfig,
    pub alerting: AlertingConfig,
    pub tracing: TracingConfig,
    pub health_check: HealthCheckConfig,
    pub resilience: ResilienceConfig,
    pub resource_monitoring: ResourceMonitoringConfig,
    pub queue_monitoring: QueueMonitoringConfig,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            circuit_breaker: CircuitBreakerConfig::default(),
            load_shedding: LoadSheddingConfig::default(),
            metrics: MetricsConfig::default(),
            alerting: AlertingConfig::default(),
            tracing: TracingConfig::default(),
            health_check: HealthCheckConfig::default(),
            resilience: ResilienceConfig::default(),
            resource_monitoring: ResourceMonitoringConfig::default(),
            queue_monitoring: QueueMonitoringConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub recovery_timeout: Duration,
    pub health_check_interval: Duration,
    pub failure_detection_window: Duration,
    pub slow_call_duration_threshold: Duration,
    pub slow_call_rate_threshold: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
            failure_detection_window: Duration::from_secs(60),
            slow_call_duration_threshold: Duration::from_secs(5),
            slow_call_rate_threshold: 0.5,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoadSheddingConfig {
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub queue_depth_threshold: usize,
    pub request_rate_threshold: usize,
    pub shedding_strategy: String,
    pub shedding_rate: f64,
    pub recovery_cooldown: Duration,
}

impl Default for LoadSheddingConfig {
    fn default() -> Self {
        Self {
            cpu_threshold: 0.8,
            memory_threshold: 0.9,
            queue_depth_threshold: 1000,
            request_rate_threshold: 1000,
            shedding_strategy: "adaptive".to_string(),
            shedding_rate: 0.1,
            recovery_cooldown: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub collection_interval: Duration,
    pub max_batch_size: usize,
    pub max_queue_size: usize,
    pub export_timeout: Duration,
    pub schedule_delay: Duration,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(15),
            max_batch_size: 512,
            max_queue_size: 2048,
            export_timeout: Duration::from_secs(30),
            schedule_delay: Duration::from_millis(5000),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlertingConfig {
    pub for_duration: Duration,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            for_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TracingConfig {
    pub sampling_ratio: f64,
    pub max_attributes_per_span: usize,
    pub max_events_per_span: usize,
    pub max_links_per_span: usize,
    pub attribute_value_length_limit: usize,
    pub attribute_count_limit: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            sampling_ratio: 0.1,
            max_attributes_per_span: 128,
            max_events_per_span: 128,
            max_links_per_span: 128,
            attribute_value_length_limit: 4096,
            attribute_count_limit: 128,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthCheckConfig {
    pub liveness_initial_delay: Duration,
    pub liveness_period: Duration,
    pub liveness_timeout: Duration,
    pub readiness_initial_delay: Duration,
    pub readiness_period: Duration,
    pub readiness_timeout: Duration,
    pub checks_timeout: Duration,
    pub checks_interval: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            liveness_initial_delay: Duration::from_secs(5),
            liveness_period: Duration::from_secs(30),
            liveness_timeout: Duration::from_secs(3),
            readiness_initial_delay: Duration::from_secs(10),
            readiness_period: Duration::from_secs(10),
            readiness_timeout: Duration::from_secs(3),
            checks_timeout: Duration::from_secs(5),
            checks_interval: Duration::from_secs(30),
            failure_threshold: 3,
            success_threshold: 1,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResilienceConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub request_timeout: Duration,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_concurrent_calls: usize,
    pub max_wait_queue_size: usize,
    pub wait_duration: Duration,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
            max_concurrent_calls: 10,
            max_wait_queue_size: 100,
            wait_duration: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceMonitoringConfig {
    pub collection_interval: Duration,
    pub aggregation: String,
    pub connection_pool_exhaustion: f64,
    pub response_time_degradation: Duration,
    pub error_rate_threshold: f64,
    pub capacity_warning: f64,
}

impl Default for ResourceMonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(30),
            aggregation: "avg".to_string(),
            connection_pool_exhaustion: 0.9,
            response_time_degradation: Duration::from_secs(5),
            error_rate_threshold: 0.05,
            capacity_warning: 0.8,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueMonitoringConfig {
    pub max_queue_depth: usize,
    pub max_processing_latency: Duration,
    pub max_consumer_lag: Duration,
    pub rebalance_alert_threshold: f64,
    pub balance_check_interval: Duration,
    pub evaluation_period: Duration,
}

impl Default for QueueMonitoringConfig {
    fn default() -> Self {
        Self {
            max_queue_depth: 10000,
            max_processing_latency: Duration::from_secs(30),
            max_consumer_lag: Duration::from_secs(300), // 5 minutes
            rebalance_alert_threshold: 0.1,
            balance_check_interval: Duration::from_secs(60),
            evaluation_period: Duration::from_secs(300), // 5 minutes
        }
    }
}

// =============================================================================
// Infrastructure Configuration
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InfraConfig {
    /// Queue retention policies
    pub queues: QueueInfraConfig,
    
    /// Environment scaling defaults
    pub scaling: ScalingDefaults,
    
    /// Default operational timeouts
    pub timeouts: InfraTimeouts,
}

impl Default for InfraConfig {
    fn default() -> Self {
        Self {
            queues: QueueInfraConfig::default(),
            scaling: ScalingDefaults::default(),
            timeouts: InfraTimeouts::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueInfraConfig {
    pub classifier_retention: String,
    pub feature_retention: String,
}

impl Default for QueueInfraConfig {
    fn default() -> Self {
        Self {
            classifier_retention: "7d".to_string(),
            feature_retention: "1d".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScalingDefaults {
    pub cooldown_period: Duration,
    pub default_timeout: Duration,
}

impl Default for ScalingDefaults {
    fn default() -> Self {
        Self {
            cooldown_period: Duration::from_secs(300), // 5 minutes
            default_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InfraTimeouts {
    pub default_activity_timeout: Duration,
    pub default_workflow_timeout: Duration,
}

impl Default for InfraTimeouts {
    fn default() -> Self {
        Self {
            default_activity_timeout: Duration::from_secs(300), // 5 minutes
            default_workflow_timeout: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Profile-Guided Optimization configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PgoConfig {
    /// Default number of workload samples to use for PGO
    pub default_sample_count: usize,
    /// Default number of warmup iterations before profiling
    pub default_warmup_iterations: usize,
    /// Default profile retention period in days
    pub default_profile_retention_days: u32,
    /// Default profile cache TTL in days
    pub default_profile_cache_ttl_days: u32,
    /// Default minimum performance improvement required (%)
    pub default_min_improvement_percent: f64,
    /// Default maximum allowed performance regression (%)
    pub default_max_regression_percent: f64,
    /// PGO profile data directory
    pub profile_data_directory: String,
    /// Whether to enable PGO by default for new systems
    pub enable_by_default: bool,
}

impl Default for PgoConfig {
    fn default() -> Self {
        Self {
            default_sample_count: 100,
            default_warmup_iterations: 10,
            default_profile_retention_days: 30,
            default_profile_cache_ttl_days: 30,
            default_min_improvement_percent: 5.0,
            default_max_regression_percent: 2.0,
            profile_data_directory: "/tmp/familiar-pgo-data".to_string(),
            enable_by_default: false,
        }
    }
}
