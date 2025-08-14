//! Loan management for commercial banking

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

use crate::errors::AstorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub loan_id: String,
    pub borrower_id: String,
    pub loan_type: LoanType,
    pub principal_amount: u64,
    pub outstanding_balance: u64,
    pub interest_rate: f64,
    pub term_months: u32,
    pub monthly_payment: u64,
    pub origination_date: DateTime<Utc>,
    pub maturity_date: DateTime<Utc>,
    pub status: LoanStatus,
    pub payment_history: Vec<LoanPayment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoanType {
    Personal,
    Mortgage,
    Business,
    Auto,
    Student,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoanStatus {
    Active,
    PaidOff,
    Defaulted,
    InForbearance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanPayment {
    pub payment_id: String,
    pub amount: u64,
    pub principal_portion: u64,
    pub interest_portion: u64,
    pub payment_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanManager {
    loans: HashMap<String, Loan>,
}

impl LoanManager {
    pub fn new() -> Self {
        Self {
            loans: HashMap::new(),
        }
    }

    /// Process a new loan application
    pub fn process_loan_application(
        &mut self,
        borrower_id: String,
        loan_type: LoanType,
        amount: u64,
        term_months: u32,
        interest_rate: f64,
    ) -> Result<String, AstorError> {
        // Credit check would happen here in production
        let loan_id = uuid::Uuid::new_v4().to_string();
        
        let monthly_payment = self.calculate_monthly_payment(amount, interest_rate, term_months);
        let maturity_date = Utc::now() + Duration::days((term_months * 30) as i64);

        let loan = Loan {
            loan_id: loan_id.clone(),
            borrower_id,
            loan_type,
            principal_amount: amount,
            outstanding_balance: amount,
            interest_rate,
            term_months,
            monthly_payment,
            origination_date: Utc::now(),
            maturity_date,
            status: LoanStatus::Active,
            payment_history: Vec::new(),
        };

        self.loans.insert(loan_id.clone(), loan);
        Ok(loan_id)
    }

    /// Make a loan payment
    pub fn make_payment(&mut self, loan_id: &str, amount: u64) -> Result<(), AstorError> {
        let loan = self.loans.get_mut(loan_id)
            .ok_or(AstorError::LoanNotFound)?;

        if loan.status != LoanStatus::Active {
            return Err(AstorError::InvalidLoanStatus);
        }

        // Calculate interest and principal portions
        let monthly_interest = (loan.outstanding_balance as f64 * loan.interest_rate / 12.0).round() as u64;
        let principal_portion = if amount > monthly_interest {
            amount - monthly_interest
        } else {
            0
        };
        let interest_portion = amount - principal_portion;

        // Create payment record
        let payment = LoanPayment {
            payment_id: uuid::Uuid::new_v4().to_string(),
            amount,
            principal_portion,
            interest_portion,
            payment_date: Utc::now(),
        };

        loan.payment_history.push(payment);
        loan.outstanding_balance = loan.outstanding_balance.saturating_sub(principal_portion);

        // Check if loan is paid off
        if loan.outstanding_balance == 0 {
            loan.status = LoanStatus::PaidOff;
        }

        Ok(())
    }

    /// Calculate monthly loan payment using amortization formula
    fn calculate_monthly_payment(&self, principal: u64, annual_rate: f64, term_months: u32) -> u64 {
        if annual_rate == 0.0 {
            return principal / term_months as u64;
        }

        let monthly_rate = annual_rate / 12.0;
        let payment = (principal as f64 * monthly_rate * (1.0 + monthly_rate).powi(term_months as i32)) 
            / ((1.0 + monthly_rate).powi(term_months as i32) - 1.0);
        payment.round() as u64
    }

    /// Get loan details
    pub fn get_loan(&self, loan_id: &str) -> Result<&Loan, AstorError> {
        self.loans.get(loan_id)
            .ok_or(AstorError::LoanNotFound)
    }

    /// List all loans for a borrower
    pub fn get_borrower_loans(&self, borrower_id: &str) -> Vec<&Loan> {
        self.loans.values()
            .filter(|loan| loan.borrower_id == borrower_id)
            .collect()
    }

    /// Mark loan as defaulted
    pub fn mark_default(&mut self, loan_id: &str) -> Result<(), AstorError> {
        let loan = self.loans.get_mut(loan_id)
            .ok_or(AstorError::LoanNotFound)?;
        
        loan.status = LoanStatus::Defaulted;
        Ok(())
    }

    /// Calculate total outstanding balance across all loans
    pub fn total_outstanding_balance(&self) -> u64 {
        self.loans.values()
            .filter(|loan| loan.status == LoanStatus::Active)
            .map(|loan| loan.outstanding_balance)
            .sum()
    }
}

impl Default for LoanManager {
    fn default() -> Self {
        Self::new()
    }
}
