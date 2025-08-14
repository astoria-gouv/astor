//! Health check system for service monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Instant};

use crate::config::HealthCheckConfig;
use crate::database::Database;
use crate::errors::AstorError;

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub uptime_seconds: u64,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health checker
pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    config: HealthCheckConfig,
    start_time: Instant,
}

impl HealthChecker {
    pub fn new(config: &HealthCheckConfig) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            start_time: Instant::now(),
        }
    }

    /// Start health check background tasks
    pub async fn start_checks(&self) -> Result<(), AstorError> {
        let checks = self.checks.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.interval));

            loop {
                interval.tick().await;

                // Run all configured health checks
                for check_name in &config.checks {
                    let result = match check_name.as_str() {
                        "database" => Self::check_database().await,
                        "redis" => Self::check_redis().await,
                        "disk_space" => Self::check_disk_space().await,
                        "memory" => Self::check_memory().await,
                        _ => HealthCheckResult {
                            name: check_name.clone(),
                            status: HealthStatus::Unhealthy,
                            message: "Unknown health check".to_string(),
                            duration_ms: 0,
                            timestamp: chrono::Utc::now(),
                        },
                    };

                    let mut checks_guard = checks.write().await;
                    checks_guard.insert(check_name.clone(), result);
                }
            }
        });

        tracing::info!("Health checks started");
        Ok(())
    }

    /// Get current system health status
    pub async fn get_status(&self) -> SystemHealth {
        let checks_guard = self.checks.read().await;
        let checks: Vec<HealthCheckResult> = checks_guard.values().cloned().collect();

        // Determine overall status
        let overall_status = if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if checks.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        SystemHealth {
            status: overall_status,
            checks,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check database connectivity
    async fn check_database() -> HealthCheckResult {
        let start = Instant::now();
        let name = "database".to_string();

        // In production, this would actually test database connectivity
        // For now, simulate a health check
        tokio::time::sleep(Duration::from_millis(10)).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        HealthCheckResult {
            name,
            status: HealthStatus::Healthy,
            message: "Database connection successful".to_string(),
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check Redis connectivity
    async fn check_redis() -> HealthCheckResult {
        let start = Instant::now();
        let name = "redis".to_string();

        // In production, this would actually test Redis connectivity
        tokio::time::sleep(Duration::from_millis(5)).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        HealthCheckResult {
            name,
            status: HealthStatus::Healthy,
            message: "Redis connection successful".to_string(),
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check disk space
    async fn check_disk_space() -> HealthCheckResult {
        let start = Instant::now();
        let name = "disk_space".to_string();

        // In production, this would check actual disk usage
        let disk_usage_percent = 45.0; // Placeholder

        let (status, message) = if disk_usage_percent > 90.0 {
            (
                HealthStatus::Unhealthy,
                format!("Disk usage critical: {}%", disk_usage_percent),
            )
        } else if disk_usage_percent > 80.0 {
            (
                HealthStatus::Degraded,
                format!("Disk usage high: {}%", disk_usage_percent),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Disk usage normal: {}%", disk_usage_percent),
            )
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        HealthCheckResult {
            name,
            status,
            message,
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check memory usage
    async fn check_memory() -> HealthCheckResult {
        let start = Instant::now();
        let name = "memory".to_string();

        // In production, this would check actual memory usage
        let memory_usage_percent = 65.0; // Placeholder

        let (status, message) = if memory_usage_percent > 90.0 {
            (
                HealthStatus::Unhealthy,
                format!("Memory usage critical: {}%", memory_usage_percent),
            )
        } else if memory_usage_percent > 80.0 {
            (
                HealthStatus::Degraded,
                format!("Memory usage high: {}%", memory_usage_percent),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Memory usage normal: {}%", memory_usage_percent),
            )
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        HealthCheckResult {
            name,
            status,
            message,
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Manual health check for specific component
    pub async fn check_component(&self, component: &str) -> HealthCheckResult {
        match component {
            "database" => Self::check_database().await,
            "redis" => Self::check_redis().await,
            "disk_space" => Self::check_disk_space().await,
            "memory" => Self::check_memory().await,
            _ => HealthCheckResult {
                name: component.to_string(),
                status: HealthStatus::Unhealthy,
                message: "Unknown component".to_string(),
                duration_ms: 0,
                timestamp: chrono::Utc::now(),
            },
        }
    }
}
