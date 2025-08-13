//! Metrics collection and Prometheus integration

use prometheus::{
    Counter, Histogram, Gauge, IntCounter, IntGauge,
    register_counter, register_histogram, register_gauge,
    register_int_counter, register_int_gauge,
    Encoder, TextEncoder, Registry,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::config::MetricsConfig;
use crate::errors::AstorError;
use super::BusinessMetric;

/// Metrics collector with Prometheus integration
pub struct MetricsCollector {
    registry: Registry,
    
    // HTTP metrics
    http_requests_total: IntCounter,
    http_request_duration: Histogram,
    http_requests_in_flight: IntGauge,
    
    // Business metrics
    transactions_total: IntCounter,
    transactions_failed: IntCounter,
    currency_issued_total: Counter,
    active_accounts: IntGauge,
    
    // System metrics
    database_connections: IntGauge,
    redis_connections: IntGauge,
    memory_usage: Gauge,
    cpu_usage: Gauge,
    
    // Security metrics
    failed_logins: IntCounter,
    security_violations: IntCounter,
    
    // Custom metrics
    custom_metrics: Arc<RwLock<HashMap<String, Box<dyn prometheus::core::Metric + Send + Sync>>>>,
    
    config: MetricsConfig,
}

impl MetricsCollector {
    pub async fn new(config: &MetricsConfig) -> Result<Self, AstorError> {
        let registry = Registry::new();
        
        // Register HTTP metrics
        let http_requests_total = register_int_counter!(
            "astor_http_requests_total",
            "Total number of HTTP requests"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let http_request_duration = register_histogram!(
            "astor_http_request_duration_seconds",
            "HTTP request duration in seconds",
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let http_requests_in_flight = register_int_gauge!(
            "astor_http_requests_in_flight",
            "Number of HTTP requests currently being processed"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        // Register business metrics
        let transactions_total = register_int_counter!(
            "astor_transactions_total",
            "Total number of transactions processed"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let transactions_failed = register_int_counter!(
            "astor_transactions_failed_total",
            "Total number of failed transactions"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let currency_issued_total = register_counter!(
            "astor_currency_issued_total",
            "Total amount of currency issued"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let active_accounts = register_int_gauge!(
            "astor_active_accounts",
            "Number of active accounts"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        // Register system metrics
        let database_connections = register_int_gauge!(
            "astor_database_connections",
            "Number of active database connections"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let redis_connections = register_int_gauge!(
            "astor_redis_connections",
            "Number of active Redis connections"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let memory_usage = register_gauge!(
            "astor_memory_usage_bytes",
            "Memory usage in bytes"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let cpu_usage = register_gauge!(
            "astor_cpu_usage_percent",
            "CPU usage percentage"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        // Register security metrics
        let failed_logins = register_int_counter!(
            "astor_failed_logins_total",
            "Total number of failed login attempts"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        let security_violations = register_int_counter!(
            "astor_security_violations_total",
            "Total number of security violations"
        ).map_err(|e| AstorError::MonitoringError(format!("Failed to register metric: {}", e)))?;
        
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_requests_in_flight,
            transactions_total,
            transactions_failed,
            currency_issued_total,
            active_accounts,
            database_connections,
            redis_connections,
            memory_usage,
            cpu_usage,
            failed_logins,
            security_violations,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
        })
    }

    /// Start metrics collection background task
    pub async fn start_collection(&self) -> Result<(), AstorError> {
        let interval_duration = Duration::from_secs(self.config.collection_interval);
        let mut interval = interval(interval_duration);
        
        // Clone necessary data for the background task
        let memory_usage = self.memory_usage.clone();
        let cpu_usage = self.cpu_usage.clone();
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                // Collect system metrics
                if let Ok(memory) = Self::get_memory_usage().await {
                    memory_usage.set(memory);
                }
                
                if let Ok(cpu) = Self::get_cpu_usage().await {
                    cpu_usage.set(cpu);
                }
            }
        });
        
        tracing::info!("Metrics collection started");
        Ok(())
    }

    /// Record business metric
    pub async fn record_business_metric(&self, metric: BusinessMetric) {
        match metric {
            BusinessMetric::TransactionCreated { amount, transaction_type } => {
                self.transactions_total.inc();
                tracing::info!("Transaction created: {} ASTOR ({})", amount, transaction_type);
            }
            BusinessMetric::TransactionCompleted { amount, duration_ms } => {
                self.http_request_duration.observe(duration_ms as f64 / 1000.0);
                tracing::info!("Transaction completed: {} ASTOR in {}ms", amount, duration_ms);
            }
            BusinessMetric::TransactionFailed { reason } => {
                self.transactions_failed.inc();
                tracing::warn!("Transaction failed: {}", reason);
            }
            BusinessMetric::CurrencyIssued { amount, issuer } => {
                self.currency_issued_total.inc_by(amount as f64);
                tracing::info!("Currency issued: {} ASTOR by {}", amount, issuer);
            }
            BusinessMetric::AccountCreated { account_type } => {
                self.active_accounts.inc();
                tracing::info!("Account created: {}", account_type);
            }
            BusinessMetric::SecurityViolation { violation_type, severity } => {
                self.security_violations.inc();
                tracing::warn!("Security violation: {} ({})", violation_type, severity);
            }
            BusinessMetric::ComplianceCheck { check_type, result } => {
                tracing::info!("Compliance check: {} = {}", check_type, result);
            }
        }
    }

    /// Record HTTP request metrics
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        self.http_requests_total.inc();
        self.http_request_duration.observe(duration.as_secs_f64());
        
        tracing::debug!(
            method = method,
            path = path,
            status = status,
            duration_ms = duration.as_millis(),
            "HTTP request completed"
        );
    }

    /// Increment in-flight requests
    pub fn inc_in_flight_requests(&self) {
        self.http_requests_in_flight.inc();
    }

    /// Decrement in-flight requests
    pub fn dec_in_flight_requests(&self) {
        self.http_requests_in_flight.dec();
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String, AstorError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        
        encoder.encode_to_string(&metric_families)
            .map_err(|e| AstorError::MonitoringError(format!("Failed to encode metrics: {}", e)))
    }

    /// Get memory usage in bytes
    async fn get_memory_usage() -> Result<f64, AstorError> {
        // In production, this would use system APIs to get actual memory usage
        // For now, return a placeholder value
        Ok(1024.0 * 1024.0 * 100.0) // 100MB placeholder
    }

    /// Get CPU usage percentage
    async fn get_cpu_usage() -> Result<f64, AstorError> {
        // In production, this would use system APIs to get actual CPU usage
        // For now, return a placeholder value
        Ok(25.0) // 25% placeholder
    }

    /// Update database connection count
    pub fn set_database_connections(&self, count: i64) {
        self.database_connections.set(count);
    }

    /// Update Redis connection count
    pub fn set_redis_connections(&self, count: i64) {
        self.redis_connections.set(count);
    }

    /// Record failed login attempt
    pub fn record_failed_login(&self) {
        self.failed_logins.inc();
    }
}
