//! Comprehensive configuration management system

pub mod environment;
pub mod secrets;
pub mod validation;
pub mod feature_flags;

use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

use crate::errors::AstorError;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub environment: Environment,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub redis: RedisConfig,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
    pub feature_flags: FeatureFlagsConfig,
    pub external_services: ExternalServicesConfig,
    pub compliance: ComplianceConfig,
}

/// Environment types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    pub fn from_string(env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "development" | "dev" => Environment::Development,
            "testing" | "test" => Environment::Testing,
            "staging" | "stage" => Environment::Staging,
            "production" | "prod" => Environment::Production,
            _ => Environment::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }
}

/// Enhanced database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub test_before_acquire: bool,
    pub migration_timeout: u64,
    pub slow_query_threshold: u64,
    pub connection_retry_attempts: u32,
    pub connection_retry_delay: u64,
}

/// Enhanced server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64,
    pub max_request_size: usize,
    pub keep_alive_timeout: u64,
    pub graceful_shutdown_timeout: u64,
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    pub enable_compression: bool,
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: Option<String>,
    pub require_client_cert: bool,
}

/// Enhanced security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
    pub bcrypt_cost: u32,
    pub max_login_attempts: u32,
    pub lockout_duration: i64,
    pub session_timeout: i64,
    pub require_mfa: bool,
    pub encryption_key: String,
    pub api_key_length: usize,
    pub password_policy: PasswordPolicyConfig,
    pub rate_limiting: RateLimitingConfig,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyConfig {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse_count: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size: u64,
    pub cleanup_interval: u64,
    pub whitelist_ips: Vec<String>,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub command_timeout: u64,
    pub retry_attempts: u32,
    pub retry_delay: u64,
    pub key_prefix: String,
    pub default_ttl: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_files: Option<u32>,
    pub structured_logging: bool,
    pub include_caller: bool,
    pub include_thread_id: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File,
    Both,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics: MetricsConfig,
    pub tracing: TracingConfig,
    pub health_check: HealthCheckConfig,
    pub alerts: AlertsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub collection_interval: u64,
    pub retention_days: u32,
    pub custom_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub jaeger_endpoint: Option<String>,
    pub sample_rate: f64,
    pub service_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub interval: u64,
    pub timeout: u64,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub email_recipients: Vec<String>,
    pub slack_webhook: Option<String>,
    pub thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub error_rate: f64,
    pub response_time_p99: u64,
    pub memory_usage: f64,
    pub cpu_usage: f64,
    pub disk_usage: f64,
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagsConfig {
    pub enabled: bool,
    pub provider: FeatureFlagProvider,
    pub refresh_interval: u64,
    pub flags: std::collections::HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureFlagProvider {
    Local,
    LaunchDarkly,
    ConfigCat,
    Custom { endpoint: String },
}

/// External services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub banking_api: Option<BankingApiConfig>,
    pub notification_service: Option<NotificationConfig>,
    pub kyc_service: Option<KycConfig>,
    pub backup_service: Option<BackupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingApiConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: u64,
    pub retry_attempts: u32,
    pub sandbox_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub email: EmailConfig,
    pub sms: Option<SmsConfig>,
    pub push: Option<PushConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub provider: String,
    pub api_key: String,
    pub from_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    pub firebase_key: String,
    pub apns_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycConfig {
    pub provider: String,
    pub api_key: String,
    pub webhook_secret: String,
    pub verification_levels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub provider: BackupProvider,
    pub schedule: String, // Cron expression
    pub retention_days: u32,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupProvider {
    S3 { bucket: String, region: String },
    GCS { bucket: String },
    Azure { container: String },
    Local { path: String },
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub audit_logging: bool,
    pub data_retention_days: u32,
    pub gdpr_compliance: bool,
    pub pci_compliance: bool,
    pub sox_compliance: bool,
    pub encryption_at_rest: bool,
    pub encryption_in_transit: bool,
    pub audit_trail_integrity: bool,
}

impl Config {
    /// Load configuration from environment and files
    pub fn load() -> Result<Self, AstorError> {
        let environment = Environment::from_string(
            &env::var("ASTOR_ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
        );

        let mut builder = config::Config::builder();

        // Load base configuration
        builder = builder.add_source(config::File::with_name("config/base").required(false));

        // Load environment-specific configuration
        let env_file = match environment {
            Environment::Development => "config/development",
            Environment::Testing => "config/testing",
            Environment::Staging => "config/staging",
            Environment::Production => "config/production",
        };
        builder = builder.add_source(config::File::with_name(env_file).required(false));

        // Load local overrides (not committed to version control)
        builder = builder.add_source(config::File::with_name("config/local").required(false));

        // Override with environment variables
        builder = builder.add_source(
            config::Environment::with_prefix("ASTOR")
                .separator("__")
                .try_parsing(true)
        );

        let settings = builder.build()
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to build config: {}", e)))?;

        let mut config: Config = settings.try_deserialize()
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to deserialize config: {}", e)))?;

        config.environment = environment;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), AstorError> {
        // Database validation
        if self.database.url.is_empty() {
            return Err(AstorError::ConfigurationError("Database URL is required".to_string()));
        }

        if self.database.max_connections == 0 {
            return Err(AstorError::ConfigurationError("Max connections must be > 0".to_string()));
        }

        // Security validation
        if self.security.jwt_secret.len() < 32 {
            return Err(AstorError::ConfigurationError("JWT secret must be at least 32 characters".to_string()));
        }

        if self.security.encryption_key.len() < 32 {
            return Err(AstorError::ConfigurationError("Encryption key must be at least 32 characters".to_string()));
        }

        // Production-specific validations
        if self.environment.is_production() {
            if self.security.jwt_secret == "default_secret" {
                return Err(AstorError::ConfigurationError("Default JWT secret not allowed in production".to_string()));
            }

            if !self.server.tls.is_some() {
                return Err(AstorError::ConfigurationError("TLS is required in production".to_string()));
            }

            if !self.compliance.encryption_at_rest {
                return Err(AstorError::ConfigurationError("Encryption at rest is required in production".to_string()));
            }
        }

        Ok(())
    }

    /// Get configuration for specific component
    pub fn get_database_config(&self) -> &DatabaseConfig {
        &self.database
    }

    pub fn get_security_config(&self) -> &SecurityConfig {
        &self.security
    }

    /// Check if feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.flags.get(feature).copied().unwrap_or(false)
    }

    /// Get environment-specific settings
    pub fn get_log_level(&self) -> &str {
        match self.environment {
            Environment::Development => "debug",
            Environment::Testing => "info",
            Environment::Staging => "info",
            Environment::Production => "warn",
        }
    }

    /// Export configuration as JSON (with secrets redacted)
    pub fn to_json_redacted(&self) -> Result<String, AstorError> {
        let mut config = self.clone();
        
        // Redact sensitive information
        config.security.jwt_secret = "[REDACTED]".to_string();
        config.security.encryption_key = "[REDACTED]".to_string();
        config.database.url = Self::redact_connection_string(&config.database.url);
        config.redis.url = Self::redact_connection_string(&config.redis.url);

        if let Some(ref mut banking) = config.external_services.banking_api {
            banking.api_key = "[REDACTED]".to_string();
        }

        serde_json::to_string_pretty(&config)
            .map_err(|e| AstorError::ConfigurationError(format!("Failed to serialize config: {}", e)))
    }

    /// Redact sensitive parts of connection strings
    fn redact_connection_string(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            let mut redacted = parsed.clone();
            if parsed.password().is_some() {
                let _ = redacted.set_password(Some("[REDACTED]"));
            }
            redacted.to_string()
        } else {
            "[REDACTED]".to_string()
        }
    }
}

/// Configuration builder for testing
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn environment(mut self, env: Environment) -> Self {
        self.config.environment = env;
        self
    }

    pub fn database_url(mut self, url: String) -> Self {
        self.config.database.url = url;
        self
    }

    pub fn jwt_secret(mut self, secret: String) -> Self {
        self.config.security.jwt_secret = secret;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            redis: RedisConfig::default(),
            logging: LoggingConfig::default(),
            monitoring: MonitoringConfig::default(),
            feature_flags: FeatureFlagsConfig::default(),
            external_services: ExternalServicesConfig::default(),
            compliance: ComplianceConfig::default(),
        }
    }
}

// Default implementations for all config structs
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/astor_dev".to_string(),
            max_connections: 20,
            min_connections: 5,
            acquire_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            test_before_acquire: true,
            migration_timeout: 300,
            slow_query_threshold: 1000,
            connection_retry_attempts: 3,
            connection_retry_delay: 1000,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cors_origins: vec!["http://localhost:3000".to_string()],
            request_timeout: 30,
            max_request_size: 1048576, // 1MB
            keep_alive_timeout: 75,
            graceful_shutdown_timeout: 30,
            worker_threads: None,
            max_blocking_threads: None,
            enable_compression: true,
            tls: None,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "development_secret_change_in_production".to_string(),
            jwt_expiration: 86400, // 24 hours
            refresh_token_expiration: 604800, // 7 days
            bcrypt_cost: 12,
            max_login_attempts: 5,
            lockout_duration: 900, // 15 minutes
            session_timeout: 3600, // 1 hour
            require_mfa: false,
            encryption_key: "development_encryption_key_32_chars".to_string(),
            api_key_length: 32,
            password_policy: PasswordPolicyConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
        }
    }
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: Some(90),
            prevent_reuse_count: 5,
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            burst_size: 20,
            window_size: 60,
            cleanup_interval: 300,
            whitelist_ips: vec!["127.0.0.1".to_string()],
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
            retry_delay: 1000,
            key_prefix: "astor:".to_string(),
            default_ttl: 3600,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Json,
            output: LogOutput::Stdout,
            file_path: None,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_files: Some(10),
            structured_logging: true,
            include_caller: false,
            include_thread_id: false,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics: MetricsConfig::default(),
            tracing: TracingConfig::default(),
            health_check: HealthCheckConfig::default(),
            alerts: AlertsConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            collection_interval: 60,
            retention_days: 30,
            custom_metrics: vec![],
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jaeger_endpoint: None,
            sample_rate: 0.1,
            service_name: "astor-currency".to_string(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/health".to_string(),
            interval: 30,
            timeout: 5,
            checks: vec!["database".to_string(), "redis".to_string()],
        }
    }
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: None,
            email_recipients: vec![],
            slack_webhook: None,
            thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate: 0.05, // 5%
            response_time_p99: 1000, // 1 second
            memory_usage: 0.8, // 80%
            cpu_usage: 0.8, // 80%
            disk_usage: 0.9, // 90%
        }
    }
}

impl Default for FeatureFlagsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: FeatureFlagProvider::Local,
            refresh_interval: 300, // 5 minutes
            flags: std::collections::HashMap::new(),
        }
    }
}

impl Default for ExternalServicesConfig {
    fn default() -> Self {
        Self {
            banking_api: None,
            notification_service: None,
            kyc_service: None,
            backup_service: None,
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            audit_logging: true,
            data_retention_days: 2555, // 7 years
            gdpr_compliance: true,
            pci_compliance: false,
            sox_compliance: false,
            encryption_at_rest: true,
            encryption_in_transit: true,
            audit_trail_integrity: true,
        }
    }
}
