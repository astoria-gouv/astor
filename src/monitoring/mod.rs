//! Comprehensive monitoring and observability system

pub mod metrics;
// pub mod tracing;
pub mod health;
// pub mod alerts;
pub mod compliance;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::MonitoringConfig;
use crate::errors::AstorError;

/// Main monitoring system
pub struct MonitoringSystem {
    metrics: metrics::MetricsCollector,
    health_checker: health::HealthChecker,
    alert_manager: alerts::AlertManager,
    compliance_monitor: compliance::ComplianceMonitor,
    config: MonitoringConfig,
}

impl MonitoringSystem {
    pub async fn new(config: MonitoringConfig) -> Result<Self, AstorError> {
        let metrics = metrics::MetricsCollector::new(&config.metrics).await?;
        let health_checker = health::HealthChecker::new(&config.health_check);
        let alert_manager = alerts::AlertManager::new(&config.alerts).await?;
        let compliance_monitor = compliance::ComplianceMonitor::new();

        Ok(Self {
            metrics,
            health_checker,
            alert_manager,
            compliance_monitor,
            config,
        })
    }

    /// Start all monitoring services
    pub async fn start(&self) -> Result<(), AstorError> {
        // Start metrics collection
        self.metrics.start_collection().await?;

        // Start health checks
        self.health_checker.start_checks().await?;

        // Start alert monitoring
        self.alert_manager.start_monitoring().await?;

        // Start compliance monitoring
        self.compliance_monitor.start_monitoring().await?;

        tracing::info!("Monitoring system started successfully");
        Ok(())
    }

    /// Record business metric
    pub async fn record_business_metric(&self, metric: BusinessMetric) {
        self.metrics.record_business_metric(metric).await;
    }

    /// Record compliance event
    pub async fn record_compliance_event(&self, event: compliance::ComplianceEvent) {
        self.compliance_monitor.record_event(event).await;
    }

    /// Get system health status
    pub async fn get_health_status(&self) -> health::HealthStatus {
        self.health_checker.get_status().await
    }
}

/// Business metrics for financial operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessMetric {
    TransactionCreated {
        amount: i64,
        transaction_type: String,
    },
    TransactionCompleted {
        amount: i64,
        duration_ms: u64,
    },
    TransactionFailed {
        reason: String,
    },
    CurrencyIssued {
        amount: i64,
        issuer: String,
    },
    AccountCreated {
        account_type: String,
    },
    SecurityViolation {
        violation_type: String,
        severity: String,
    },
    ComplianceCheck {
        check_type: String,
        result: bool,
    },
}
