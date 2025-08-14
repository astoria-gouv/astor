//! Credit line management for commercial banking

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::errors::AstorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditLine {
    pub credit_line_id: String,
    pub customer_id: String,
    pub credit_limit: u64,
    pub available_credit: u64,
    pub outstanding_balance: u64,
    pub interest_rate: f64,
    pub minimum_payment: u64,
    pub status: CreditStatus,
    pub opened_date: DateTime<Utc>,
    pub last_statement_date: DateTime<Utc>,
    pub transaction_history: Vec<CreditTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreditStatus {
    Active,
    Suspended,
    Closed,
    OverLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub transaction_id: String,
    pub transaction_type: CreditTransactionType,
    pub amount: u64,
    pub description: String,
    pub transaction_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreditTransactionType {
    Purchase,
    Payment,
    Interest,
    Fee,
    CashAdvance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditManager {
    credit_lines: HashMap<String, CreditLine>,
}

impl CreditManager {
    pub fn new() -> Self {
        Self {
            credit_lines: HashMap::new(),
        }
    }

    /// Open a new credit line
    pub fn open_credit_line(
        &mut self,
        customer_id: String,
        credit_limit: u64,
        interest_rate: f64,
    ) -> Result<String, AstorError> {
        let credit_line_id = uuid::Uuid::new_v4().to_string();
        
        let credit_line = CreditLine {
            credit_line_id: credit_line_id.clone(),
            customer_id,
            credit_limit,
            available_credit: credit_limit,
            outstanding_balance: 0,
            interest_rate,
            minimum_payment: 0,
            status: CreditStatus::Active,
            opened_date: Utc::now(),
            last_statement_date: Utc::now(),
            transaction_history: Vec::new(),
        };

        self.credit_lines.insert(credit_line_id.clone(), credit_line);
        Ok(credit_line_id)
    }

    /// Make a purchase on credit
    pub fn make_purchase(
        &mut self,
        credit_line_id: &str,
        amount: u64,
        description: String,
    ) -> Result<(), AstorError> {
        let credit_line = self.credit_lines.get_mut(credit_line_id)
            .ok_or(AstorError::CreditLineNotFound)?;

        if credit_line.status != CreditStatus::Active {
            return Err(AstorError::InvalidCreditStatus);
        }

        if amount > credit_line.available_credit {
            return Err(AstorError::CreditLimitExceeded);
        }

        // Create transaction record
        let transaction = CreditTransaction {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            transaction_type: CreditTransactionType::Purchase,
            amount,
            description,
            transaction_date: Utc::now(),
        };

        credit_line.transaction_history.push(transaction);
        credit_line.outstanding_balance += amount;
        credit_line.available_credit -= amount;

        // Update minimum payment (typically 2% of balance)
        credit_line.minimum_payment = (credit_line.outstanding_balance as f64 * 0.02).round() as u64;

        // Check if over limit
        if credit_line.outstanding_balance > credit_line.credit_limit {
            credit_line.status = CreditStatus::OverLimit;
        }

        Ok(())
    }

    /// Make a payment on credit line
    pub fn make_payment(
        &mut self,
        credit_line_id: &str,
        amount: u64,
    ) -> Result<(), AstorError> {
        let credit_line = self.credit_lines.get_mut(credit_line_id)
            .ok_or(AstorError::CreditLineNotFound)?;

        let payment_amount = std::cmp::min(amount, credit_line.outstanding_balance);

        // Create payment transaction
        let transaction = CreditTransaction {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            transaction_type: CreditTransactionType::Payment,
            amount: payment_amount,
            description: "Payment".to_string(),
            transaction_date: Utc::now(),
        };

        credit_line.transaction_history.push(transaction);
        credit_line.outstanding_balance -= payment_amount;
        credit_line.available_credit += payment_amount;

        // Update minimum payment
        credit_line.minimum_payment = (credit_line.outstanding_balance as f64 * 0.02).round() as u64;

        // Update status if back under limit
        if credit_line.status == CreditStatus::OverLimit && 
           credit_line.outstanding_balance <= credit_line.credit_limit {
            credit_line.status = CreditStatus::Active;
        }

        Ok(())
    }

    /// Apply monthly interest charges
    pub fn apply_interest(&mut self, credit_line_id: &str) -> Result<u64, AstorError> {
        let credit_line = self.credit_lines.get_mut(credit_line_id)
            .ok_or(AstorError::CreditLineNotFound)?;

        if credit_line.outstanding_balance == 0 {
            return Ok(0);
        }

        let monthly_interest = (credit_line.outstanding_balance as f64 * credit_line.interest_rate / 12.0).round() as u64;

        // Create interest transaction
        let transaction = CreditTransaction {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            transaction_type: CreditTransactionType::Interest,
            amount: monthly_interest,
            description: "Monthly Interest".to_string(),
            transaction_date: Utc::now(),
        };

        credit_line.transaction_history.push(transaction);
        credit_line.outstanding_balance += monthly_interest;
        credit_line.available_credit = credit_line.credit_limit.saturating_sub(credit_line.outstanding_balance);

        // Update minimum payment
        credit_line.minimum_payment = (credit_line.outstanding_balance as f64 * 0.02).round() as u64;

        Ok(monthly_interest)
    }

    /// Get credit line details
    pub fn get_credit_line(&self, credit_line_id: &str) -> Result<&CreditLine, AstorError> {
        self.credit_lines.get(credit_line_id)
            .ok_or(AstorError::CreditLineNotFound)
    }

    /// List all credit lines for a customer
    pub fn get_customer_credit_lines(&self, customer_id: &str) -> Vec<&CreditLine> {
        self.credit_lines.values()
            .filter(|credit_line| credit_line.customer_id == customer_id)
            .collect()
    }

    /// Close a credit line
    pub fn close_credit_line(&mut self, credit_line_id: &str) -> Result<(), AstorError> {
        let credit_line = self.credit_lines.get_mut(credit_line_id)
            .ok_or(AstorError::CreditLineNotFound)?;

        if credit_line.outstanding_balance > 0 {
            return Err(AstorError::OutstandingBalance);
        }

        credit_line.status = CreditStatus::Closed;
        Ok(())
    }

    /// Calculate total credit exposure
    pub fn total_outstanding_balance(&self) -> u64 {
        self.credit_lines.values()
            .filter(|credit_line| matches!(credit_line.status, CreditStatus::Active | CreditStatus::OverLimit))
            .map(|credit_line| credit_line.outstanding_balance)
            .sum()
    }
}

impl Default for CreditManager {
    fn default() -> Self {
        Self::new()
    }
}
