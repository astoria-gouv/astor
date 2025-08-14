//! Security audit logging and compliance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::errors::AstorError;

/// Security events that need to be audited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    LoginAttempt {
        user_id: String,
        ip_address: String,
        success: bool,
        timestamp: DateTime<Utc>,
        user_agent: Option<String>,
    },
    PermissionDenied {
        user_id: String,
        resource: String,
        action: String,
        timestamp: DateTime<Utc>,
    },
    HighRiskOperation {
        user_id: String,
        operation: String,
        risk_score: f64,
        ip_address: String,
    },
    AdminAction {
        admin_id: String,
        action: String,
        target: String,
        timestamp: DateTime<Utc>,
    },
    SecurityViolation {
        user_id: Option<String>,
        violation_type: String,
        details: String,
        ip_address: String,
        timestamp: DateTime<Utc>,
    },
    DataAccess {
        user_id: String,
        resource_type: String,
        resource_id: String,
        action: String,
        timestamp: DateTime<Utc>,
    },
    SystemEvent {
        event_type: String,
        details: String,
        timestamp: DateTime<Utc>,
    },
}

/// Audit log entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event: SecurityEvent,
    pub severity: AuditSeverity,
    pub source: String,
    pub correlation_id: Option<Uuid>,
    pub metadata: serde_json::Value,
}

/// Severity levels for audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Security audit logger
pub struct SecurityAuditLogger {
    logs: VecDeque<AuditLogEntry>,
    max_logs: usize,
    alert_thresholds: std::collections::HashMap<String, u32>,
}

impl SecurityAuditLogger {
    pub fn new() -> Self {
        let mut alert_thresholds = std::collections::HashMap::new();
        alert_thresholds.insert("failed_login".to_string(), 5);
        alert_thresholds.insert("permission_denied".to_string(), 10);
        alert_thresholds.insert("high_risk_operation".to_string(), 3);

        Self {
            logs: VecDeque::new(),
            max_logs: 10000, // Keep last 10k logs in memory
            alert_thresholds,
        }
    }

    /// Log a security event
    pub async fn log_security_event(&mut self, event: SecurityEvent) -> Result<(), AstorError> {
        let severity = self.determine_severity(&event);
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            event: event.clone(),
            severity,
            source: "astor-security".to_string(),
            correlation_id: None,
            metadata: serde_json::json!({}),
        };

        // Add to in-memory log
        self.logs.push_back(entry.clone());

        // Maintain max size
        if self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }

        // Check for alert conditions
        self.check_alert_conditions(&event).await?;

        // In production, this would also:
        // - Write to persistent storage (database, file, SIEM)
        // - Send to monitoring systems
        // - Trigger alerts for critical events

        tracing::info!("Security event logged: {:?}", entry);

        Ok(())
    }

    /// Determine severity of security event
    fn determine_severity(&self, event: &SecurityEvent) -> AuditSeverity {
        match event {
            SecurityEvent::LoginAttempt { success: false, .. } => AuditSeverity::Warning,
            SecurityEvent::PermissionDenied { .. } => AuditSeverity::Warning,
            SecurityEvent::HighRiskOperation { risk_score, .. } => {
                if *risk_score > 0.8 {
                    AuditSeverity::Critical
                } else {
                    AuditSeverity::Warning
                }
            }
            SecurityEvent::SecurityViolation { .. } => AuditSeverity::Error,
            SecurityEvent::AdminAction { .. } => AuditSeverity::Info,
            SecurityEvent::DataAccess { .. } => AuditSeverity::Info,
            SecurityEvent::SystemEvent { .. } => AuditSeverity::Info,
        }
    }

    /// Check if event should trigger alerts
    async fn check_alert_conditions(&self, event: &SecurityEvent) -> Result<(), AstorError> {
        let event_type = match event {
            SecurityEvent::LoginAttempt { success: false, .. } => "failed_login",
            SecurityEvent::PermissionDenied { .. } => "permission_denied",
            SecurityEvent::HighRiskOperation { .. } => "high_risk_operation",
            _ => return Ok(()),
        };

        // Count recent events of this type
        let recent_count = self
            .logs
            .iter()
            .rev()
            .take(100) // Check last 100 events
            .filter(|entry| {
                // Check if event matches type and is recent (last hour)
                let is_recent = match &entry.event {
                    SecurityEvent::LoginAttempt {
                        timestamp,
                        success: false,
                        ..
                    } => {
                        event_type == "failed_login"
                            && Utc::now() - *timestamp < chrono::Duration::hours(1)
                    }
                    SecurityEvent::PermissionDenied { timestamp, .. } => {
                        event_type == "permission_denied"
                            && Utc::now() - *timestamp < chrono::Duration::hours(1)
                    }
                    SecurityEvent::HighRiskOperation { .. } => event_type == "high_risk_operation",
                    _ => false,
                };
                is_recent
            })
            .count();

        if let Some(&threshold) = self.alert_thresholds.get(event_type) {
            if recent_count >= threshold as usize {
                // In production, this would trigger alerts via:
                // - Email notifications
                // - Slack/Teams messages
                // - PagerDuty incidents
                // - SIEM system alerts
                tracing::warn!(
                    "Alert threshold exceeded for {}: {} events in last hour (threshold: {})",
                    event_type,
                    recent_count,
                    threshold
                );
            }
        }

        Ok(())
    }

    /// Get audit logs with filtering
    pub fn get_logs(
        &self,
        severity_filter: Option<AuditSeverity>,
        limit: Option<usize>,
    ) -> Vec<&AuditLogEntry> {
        let mut filtered: Vec<&AuditLogEntry> = self
            .logs
            .iter()
            .filter(|entry| {
                severity_filter
                    .as_ref()
                    .map_or(true, |s| entry.severity >= *s)
            })
            .collect();

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| {
            let a_time = match &a.event {
                SecurityEvent::LoginAttempt { timestamp, .. } => *timestamp,
                SecurityEvent::PermissionDenied { timestamp, .. } => *timestamp,
                SecurityEvent::AdminAction { timestamp, .. } => *timestamp,
                SecurityEvent::SecurityViolation { timestamp, .. } => *timestamp,
                SecurityEvent::DataAccess { timestamp, .. } => *timestamp,
                SecurityEvent::SystemEvent { timestamp, .. } => *timestamp,
                _ => Utc::now(),
            };
            let b_time = match &b.event {
                SecurityEvent::LoginAttempt { timestamp, .. } => *timestamp,
                SecurityEvent::PermissionDenied { timestamp, .. } => *timestamp,
                SecurityEvent::AdminAction { timestamp, .. } => *timestamp,
                SecurityEvent::SecurityViolation { timestamp, .. } => *timestamp,
                SecurityEvent::DataAccess { timestamp, .. } => *timestamp,
                SecurityEvent::SystemEvent { timestamp, .. } => *timestamp,
                _ => Utc::now(),
            };
            b_time.cmp(&a_time)
        });

        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Generate compliance report
    pub fn generate_compliance_report(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ComplianceReport {
        let relevant_logs: Vec<&AuditLogEntry> = self
            .logs
            .iter()
            .filter(|entry| {
                let event_time = match &entry.event {
                    SecurityEvent::LoginAttempt { timestamp, .. } => *timestamp,
                    SecurityEvent::PermissionDenied { timestamp, .. } => *timestamp,
                    SecurityEvent::AdminAction { timestamp, .. } => *timestamp,
                    SecurityEvent::SecurityViolation { timestamp, .. } => *timestamp,
                    SecurityEvent::DataAccess { timestamp, .. } => *timestamp,
                    SecurityEvent::SystemEvent { timestamp, .. } => *timestamp,
                    _ => Utc::now(),
                };
                event_time >= start_date && event_time <= end_date
            })
            .collect();

        ComplianceReport::new(relevant_logs, start_date, end_date)
    }
}

/// Compliance report for regulatory requirements
#[derive(Debug, Serialize)]
pub struct ComplianceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub login_attempts: usize,
    pub failed_logins: usize,
    pub admin_actions: usize,
    pub security_violations: usize,
    pub high_risk_operations: usize,
    pub data_access_events: usize,
    pub generated_at: DateTime<Utc>,
}

impl ComplianceReport {
    fn new(logs: Vec<&AuditLogEntry>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let mut login_attempts = 0;
        let mut failed_logins = 0;
        let mut admin_actions = 0;
        let mut security_violations = 0;
        let mut high_risk_operations = 0;
        let mut data_access_events = 0;

        for entry in &logs {
            match &entry.event {
                SecurityEvent::LoginAttempt { success, .. } => {
                    login_attempts += 1;
                    if !success {
                        failed_logins += 1;
                    }
                }
                SecurityEvent::AdminAction { .. } => admin_actions += 1,
                SecurityEvent::SecurityViolation { .. } => security_violations += 1,
                SecurityEvent::HighRiskOperation { .. } => high_risk_operations += 1,
                SecurityEvent::DataAccess { .. } => data_access_events += 1,
                _ => {}
            }
        }

        Self {
            period_start: start,
            period_end: end,
            total_events: logs.len(),
            login_attempts,
            failed_logins,
            admin_actions,
            security_violations,
            high_risk_operations,
            data_access_events,
            generated_at: Utc::now(),
        }
    }
}
