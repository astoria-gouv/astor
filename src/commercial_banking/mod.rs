//! Commercial banking operations - loans, deposits, credit

// pub mod loans;
// pub mod deposits;
// pub mod credit;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::AstorError;

/// Commercial bank
pub struct CommercialBank {
    pub bank_id: String,
    pub bank_name: String,
    deposits: HashMap<String, DepositAccount>,
    loans: HashMap<String, Loan>,
    credit_lines: HashMap<String, CreditLine>,
    reserve_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAccount {
    pub account_id: String,
    pub customer_id: String,
    pub account_type: DepositAccountType,
    pub balance: u64,
    pub interest_rate: f64,
    pub opened_date: DateTime<Utc>,
    pub last_interest_payment: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepositAccountType {
    Checking,
    Savings,
    TimeDeposit { maturity_date: DateTime<Utc> },
    MoneyMarket,
}

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
pub struct CreditLine {
    pub credit_line_id: String,
    pub customer_id: String,
    pub credit_limit: u64,
    pub available_credit: u64,
    pub interest_rate: f64,
    pub minimum_payment: u64,
    pub status: CreditStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreditStatus {
    Active,
    Suspended,
    Closed,
}

impl CommercialBank {
    pub fn new(bank_id: String, bank_name: String) -> Self {
        Self {
            bank_id,
            bank_name,
            deposits: HashMap::new(),
            loans: HashMap::new(),
            credit_lines: HashMap::new(),
            reserve_balance: 0,
        }
    }

    /// Open deposit account
    pub fn open_deposit_account(
        &mut self,
        customer_id: String,
        account_type: DepositAccountType,
        initial_deposit: u64,
        interest_rate: f64,
    ) -> Result<String, AstorError> {
        let account_id = uuid::Uuid::new_v4().to_string();

        let account = DepositAccount {
            account_id: account_id.clone(),
            customer_id,
            account_type,
            balance: initial_deposit,
            interest_rate,
            opened_date: Utc::now(),
            last_interest_payment: Utc::now(),
        };

        self.deposits.insert(account_id.clone(), account);
        Ok(account_id)
    }

    /// Process loan application
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
        };

        self.loans.insert(loan_id.clone(), loan);
        Ok(loan_id)
    }

    /// Calculate monthly loan payment
    fn calculate_monthly_payment(&self, principal: u64, annual_rate: f64, term_months: u32) -> u64 {
        let monthly_rate = annual_rate / 12.0;
        let payment =
            (principal as f64 * monthly_rate * (1.0 + monthly_rate).powi(term_months as i32))
                / ((1.0 + monthly_rate).powi(term_months as i32) - 1.0);
        payment.round() as u64
    }

    /// Pay interest on deposits
    pub fn pay_deposit_interest(&mut self) -> Result<u64, AstorError> {
        let mut total_interest_paid = 0u64;

        for account in self.deposits.values_mut() {
            let days_since_last_payment = (Utc::now() - account.last_interest_payment).num_days();
            if days_since_last_payment >= 30 {
                // Monthly interest
                let interest =
                    (account.balance as f64 * account.interest_rate / 12.0).round() as u64;
                account.balance += interest;
                account.last_interest_payment = Utc::now();
                total_interest_paid += interest;
            }
        }

        Ok(total_interest_paid)
    }
}
