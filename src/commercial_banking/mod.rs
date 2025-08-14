//! Commercial banking operations - loans, deposits, credit

pub mod loans;
pub mod deposits;
pub mod credit;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

use crate::errors::AstorError;
use self::{
    deposits::{DepositManager, DepositAccount, DepositAccountType},
    loans::{LoanManager, Loan, LoanType, LoanStatus},
    credit::{CreditManager, CreditLine, CreditStatus},
};

/// Commercial bank with integrated deposit, loan, and credit management
pub struct CommercialBank {
    pub bank_id: String,
    pub bank_name: String,
    pub deposit_manager: DepositManager,
    pub loan_manager: LoanManager,
    pub credit_manager: CreditManager,
    reserve_balance: u64,
}

impl CommercialBank {
    pub fn new(bank_id: String, bank_name: String) -> Self {
        Self {
            bank_id,
            bank_name,
            deposit_manager: DepositManager::new(),
            loan_manager: LoanManager::new(),
            credit_manager: CreditManager::new(),
            reserve_balance: 0,
        }
    }

    /// Get total bank assets
    pub fn total_assets(&self) -> u64 {
        self.reserve_balance + self.loan_manager.total_outstanding_balance()
    }

    /// Get total bank liabilities
    pub fn total_liabilities(&self) -> u64 {
        // This would include all deposit balances in a real implementation
        self.credit_manager.total_outstanding_balance()
    }

    /// Set reserve balance
    pub fn set_reserve_balance(&mut self, balance: u64) {
        self.reserve_balance = balance;
    }

    /// Get reserve balance
    pub fn get_reserve_balance(&self) -> u64 {
        self.reserve_balance
    }
}