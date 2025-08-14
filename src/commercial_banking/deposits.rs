//! Deposit account management for commercial banking

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

use crate::errors::AstorError;

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
pub struct DepositManager {
    deposits: HashMap<String, DepositAccount>,
}

impl DepositManager {
    pub fn new() -> Self {
        Self {
            deposits: HashMap::new(),
        }
    }

    /// Open a new deposit account
    pub fn open_account(
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

    /// Make a deposit to an existing account
    pub fn make_deposit(&mut self, account_id: &str, amount: u64) -> Result<(), AstorError> {
        let account = self.deposits.get_mut(account_id)
            .ok_or(AstorError::AccountNotFound)?;
        
        account.balance += amount;
        Ok(())
    }

    /// Make a withdrawal from an account
    pub fn make_withdrawal(&mut self, account_id: &str, amount: u64) -> Result<(), AstorError> {
        let account = self.deposits.get_mut(account_id)
            .ok_or(AstorError::AccountNotFound)?;
        
        if account.balance < amount {
            return Err(AstorError::InsufficientFunds);
        }
        
        account.balance -= amount;
        Ok(())
    }

    /// Get account balance
    pub fn get_balance(&self, account_id: &str) -> Result<u64, AstorError> {
        let account = self.deposits.get(account_id)
            .ok_or(AstorError::AccountNotFound)?;
        
        Ok(account.balance)
    }

    /// Pay interest on all eligible accounts
    pub fn pay_interest(&mut self) -> Result<u64, AstorError> {
        let mut total_interest_paid = 0u64;
        
        for account in self.deposits.values_mut() {
            let days_since_last_payment = (Utc::now() - account.last_interest_payment).num_days();
            if days_since_last_payment >= 30 { // Monthly interest
                let interest = (account.balance as f64 * account.interest_rate / 12.0).round() as u64;
                account.balance += interest;
                account.last_interest_payment = Utc::now();
                total_interest_paid += interest;
            }
        }
        
        Ok(total_interest_paid)
    }

    /// Close an account
    pub fn close_account(&mut self, account_id: &str) -> Result<u64, AstorError> {
        let account = self.deposits.remove(account_id)
            .ok_or(AstorError::AccountNotFound)?;
        
        Ok(account.balance)
    }

    /// Get account details
    pub fn get_account(&self, account_id: &str) -> Result<&DepositAccount, AstorError> {
        self.deposits.get(account_id)
            .ok_or(AstorError::AccountNotFound)
    }

    /// List all accounts for a customer
    pub fn get_customer_accounts(&self, customer_id: &str) -> Vec<&DepositAccount> {
        self.deposits.values()
            .filter(|account| account.customer_id == customer_id)
            .collect()
    }
}

impl Default for DepositManager {
    fn default() -> Self {
        Self::new()
    }
}
