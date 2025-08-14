//! Central banking functions for monetary policy and currency management

// pub mod monetary_policy;
// pub mod reserve_management;
// pub mod interest_rates;
// pub mod money_supply;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::AstorError;

/// Central bank configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralBankConfig {
    pub base_interest_rate: f64,
    pub reserve_requirement_ratio: f64,
    pub inflation_target: f64,
    pub money_supply_growth_target: f64,
    pub emergency_lending_rate: f64,
}

/// Central bank operations
pub struct CentralBank {
    config: CentralBankConfig,
    total_money_supply: u64,
    reserve_balances: HashMap<String, u64>, // Bank ID -> Reserve Balance
    interest_rates: HashMap<String, f64>,   // Rate type -> Rate
    monetary_policy_decisions: Vec<MonetaryPolicyDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryPolicyDecision {
    pub decision_id: String,
    pub decision_type: PolicyDecisionType,
    pub effective_date: DateTime<Utc>,
    pub rationale: String,
    pub impact_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecisionType {
    InterestRateChange {
        old_rate: f64,
        new_rate: f64,
    },
    ReserveRequirementChange {
        old_ratio: f64,
        new_ratio: f64,
    },
    MoneySupplyAdjustment {
        amount: i64,
    }, // Positive = increase, negative = decrease
    EmergencyMeasure {
        measure_type: String,
        details: String,
    },
}

impl CentralBank {
    pub fn new(config: CentralBankConfig) -> Self {
        let mut interest_rates = HashMap::new();
        interest_rates.insert("base_rate".to_string(), config.base_interest_rate);
        interest_rates.insert("emergency_rate".to_string(), config.emergency_lending_rate);
        interest_rates.insert("deposit_rate".to_string(), config.base_interest_rate - 0.5);

        Self {
            config,
            total_money_supply: 0,
            reserve_balances: HashMap::new(),
            interest_rates,
            monetary_policy_decisions: Vec::new(),
        }
    }

    /// Issue new currency (monetary expansion)
    pub fn issue_currency(
        &mut self,
        amount: u64,
        justification: String,
    ) -> Result<String, AstorError> {
        let decision = MonetaryPolicyDecision {
            decision_id: uuid::Uuid::new_v4().to_string(),
            decision_type: PolicyDecisionType::MoneySupplyAdjustment {
                amount: amount as i64,
            },
            effective_date: Utc::now(),
            rationale: justification,
            impact_assessment: format!("Money supply increased by {} ASTOR", amount),
        };

        self.total_money_supply = self
            .total_money_supply
            .checked_add(amount)
            .ok_or_else(|| AstorError::CentralBankError("Money supply overflow".to_string()))?;

        self.monetary_policy_decisions.push(decision.clone());
        Ok(decision.decision_id)
    }

    /// Set interest rates
    pub fn set_interest_rate(
        &mut self,
        rate_type: String,
        new_rate: f64,
        justification: String,
    ) -> Result<(), AstorError> {
        let old_rate = self.interest_rates.get(&rate_type).copied().unwrap_or(0.0);

        let decision = MonetaryPolicyDecision {
            decision_id: uuid::Uuid::new_v4().to_string(),
            decision_type: PolicyDecisionType::InterestRateChange { old_rate, new_rate },
            effective_date: Utc::now(),
            rationale: justification,
            impact_assessment: format!(
                "{} rate changed from {}% to {}%",
                rate_type,
                old_rate * 100.0,
                new_rate * 100.0
            ),
        };

        self.interest_rates.insert(rate_type, new_rate);
        self.monetary_policy_decisions.push(decision);
        Ok(())
    }

    /// Manage bank reserves
    pub fn set_bank_reserves(&mut self, bank_id: String, amount: u64) -> Result<(), AstorError> {
        self.reserve_balances.insert(bank_id, amount);
        Ok(())
    }

    /// Get current interest rate
    pub fn get_interest_rate(&self, rate_type: &str) -> Option<f64> {
        self.interest_rates.get(rate_type).copied()
    }

    /// Get money supply statistics
    pub fn get_money_supply_stats(&self) -> MoneySupplyStats {
        MoneySupplyStats {
            total_supply: self.total_money_supply,
            reserve_balances: self.reserve_balances.clone(),
            base_interest_rate: self.config.base_interest_rate,
            inflation_target: self.config.inflation_target,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneySupplyStats {
    pub total_supply: u64,
    pub reserve_balances: HashMap<String, u64>,
    pub base_interest_rate: f64,
    pub inflation_target: f64,
}
