//! Fraud detection and risk assessment

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AstorError;

/// Risk score for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    score: f64, // 0.0 to 1.0
    factors: Vec<RiskFactor>,
    timestamp: DateTime<Utc>,
}

impl RiskScore {
    pub fn new(score: f64, factors: Vec<RiskFactor>) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
            factors,
            timestamp: Utc::now(),
        }
    }

    pub fn score(&self) -> f64 {
        self.score
    }

    pub fn is_high_risk(&self) -> bool {
        self.score > 0.7
    }

    pub fn is_medium_risk(&self) -> bool {
        self.score > 0.4 && self.score <= 0.7
    }

    pub fn is_low_risk(&self) -> bool {
        self.score <= 0.4
    }
}

/// Risk factors that contribute to overall risk score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactor {
    UnusualTransactionAmount {
        amount: i64,
        typical_range: (i64, i64),
    },
    UnusualTransactionFrequency {
        count: u32,
        time_window: Duration,
    },
    NewIpAddress {
        ip: String,
    },
    UnusualTimeOfDay {
        hour: u32,
    },
    GeographicAnomaly {
        country: String,
        typical_countries: Vec<String>,
    },
    VelocityCheck {
        transactions_per_hour: u32,
    },
    AccountAge {
        days: i64,
    },
    SuspiciousPattern {
        pattern: String,
    },
}

/// Transaction pattern for analysis
#[derive(Debug, Clone)]
pub struct TransactionPattern {
    pub user_id: String,
    pub amount: i64,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub transaction_type: String,
}

/// Fraud detection engine
pub struct FraudDetector {
    transaction_history: HashMap<String, Vec<TransactionPattern>>,
    ip_reputation: HashMap<String, f64>,
    user_profiles: HashMap<String, UserProfile>,
}

#[derive(Debug, Clone)]
struct UserProfile {
    typical_transaction_amounts: Vec<i64>,
    typical_transaction_times: Vec<u32>, // Hours of day
    typical_ips: Vec<String>,
    account_created: DateTime<Utc>,
    total_transactions: u32,
}

impl FraudDetector {
    pub fn new() -> Self {
        Self {
            transaction_history: HashMap::new(),
            ip_reputation: HashMap::new(),
            user_profiles: HashMap::new(),
        }
    }

    /// Assess risk for a transaction
    pub async fn assess_risk(
        &mut self,
        user_id: &str,
        operation: &str,
        ip_address: &str,
    ) -> Result<RiskScore, AstorError> {
        let mut risk_factors = Vec::new();
        let mut total_risk = 0.0;

        // Check IP reputation
        let ip_risk = self.check_ip_reputation(ip_address);
        if ip_risk > 0.5 {
            risk_factors.push(RiskFactor::NewIpAddress {
                ip: ip_address.to_string(),
            });
            total_risk += ip_risk * 0.3;
        }

        // Check user profile if exists
        if let Some(profile) = self.user_profiles.get(user_id) {
            // Check account age
            let account_age = Utc::now() - profile.account_created;
            if account_age.num_days() < 7 {
                risk_factors.push(RiskFactor::AccountAge {
                    days: account_age.num_days(),
                });
                total_risk += 0.2;
            }

            // Check transaction velocity
            let recent_transactions = self.get_recent_transactions(user_id, Duration::hours(1));
            if recent_transactions.len() > 10 {
                risk_factors.push(RiskFactor::VelocityCheck {
                    transactions_per_hour: recent_transactions.len() as u32,
                });
                total_risk += 0.4;
            }

            // Check time of day patterns
            let current_hour = Utc::now().hour();
            if !profile.typical_transaction_times.contains(&current_hour) {
                risk_factors.push(RiskFactor::UnusualTimeOfDay { hour: current_hour });
                total_risk += 0.1;
            }
        } else {
            // New user - higher risk
            total_risk += 0.3;
        }

        // Check for suspicious patterns
        if self.detect_suspicious_patterns(user_id, operation).await? {
            risk_factors.push(RiskFactor::SuspiciousPattern {
                pattern: "Rapid sequential transactions".to_string(),
            });
            total_risk += 0.5;
        }

        Ok(RiskScore::new(total_risk.min(1.0), risk_factors))
    }

    /// Record transaction for pattern analysis
    pub fn record_transaction(&mut self, pattern: TransactionPattern) {
        // Update transaction history
        self.transaction_history
            .entry(pattern.user_id.clone())
            .or_insert_with(Vec::new)
            .push(pattern.clone());

        // Update user profile
        let profile = self
            .user_profiles
            .entry(pattern.user_id.clone())
            .or_insert_with(|| UserProfile {
                typical_transaction_amounts: Vec::new(),
                typical_transaction_times: Vec::new(),
                typical_ips: Vec::new(),
                account_created: Utc::now(),
                total_transactions: 0,
            });

        profile.typical_transaction_amounts.push(pattern.amount);
        profile
            .typical_transaction_times
            .push(pattern.timestamp.hour());
        if !profile.typical_ips.contains(&pattern.ip_address) {
            profile.typical_ips.push(pattern.ip_address.clone());
        }
        profile.total_transactions += 1;

        // Keep only recent data to avoid memory bloat
        if profile.typical_transaction_amounts.len() > 100 {
            profile.typical_transaction_amounts.drain(0..50);
        }
    }

    /// Check IP reputation
    fn check_ip_reputation(&mut self, ip_address: &str) -> f64 {
        // In production, this would check against threat intelligence feeds
        *self.ip_reputation.get(ip_address).unwrap_or(&0.0)
    }

    /// Get recent transactions for user
    fn get_recent_transactions(
        &self,
        user_id: &str,
        duration: Duration,
    ) -> Vec<&TransactionPattern> {
        let cutoff = Utc::now() - duration;

        self.transaction_history
            .get(user_id)
            .map(|transactions| {
                transactions
                    .iter()
                    .filter(|t| t.timestamp > cutoff)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Detect suspicious patterns
    async fn detect_suspicious_patterns(
        &self,
        user_id: &str,
        operation: &str,
    ) -> Result<bool, AstorError> {
        if let Some(transactions) = self.transaction_history.get(user_id) {
            let recent = transactions
                .iter()
                .filter(|t| Utc::now() - t.timestamp < Duration::minutes(5))
                .count();

            // Flag if more than 5 transactions in 5 minutes
            if recent > 5 {
                return Ok(true);
            }

            // Check for round number patterns (potential money laundering)
            let round_amounts = transactions
                .iter()
                .filter(|t| t.amount % 1000 == 0 && t.amount > 10000)
                .count();

            if round_amounts > 3 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Update IP reputation based on behavior
    pub fn update_ip_reputation(&mut self, ip_address: &str, reputation_delta: f64) {
        let current = self.ip_reputation.get(ip_address).unwrap_or(&0.5);
        let new_reputation = (current + reputation_delta).clamp(0.0, 1.0);
        self.ip_reputation
            .insert(ip_address.to_string(), new_reputation);
    }
}

/// Machine learning-based anomaly detection (placeholder for production ML models)
pub struct AnomalyDetector {
    // In production, this would contain trained ML models
    baseline_metrics: HashMap<String, f64>,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            baseline_metrics: HashMap::new(),
        }
    }

    /// Detect anomalies in transaction patterns
    pub fn detect_anomaly(&self, _transaction: &TransactionPattern) -> f64 {
        // Placeholder - in production this would use trained models
        // to detect statistical anomalies in transaction patterns
        0.0
    }

    /// Update baseline metrics
    pub fn update_baseline(&mut self, metric_name: &str, value: f64) {
        self.baseline_metrics.insert(metric_name.to_string(), value);
    }
}
