//! User account management module

use chrono::{DateTime, Utc};
use ed25519_dalek::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AstorError;
use crate::security::Signature;

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub public_key: Option<PublicKey>,
    pub balance: u64,
    pub created_at: DateTime<Utc>,
    pub last_transaction: Option<DateTime<Utc>>,
    pub is_frozen: bool,
}

/// Manages user accounts and balances
pub struct AccountManager {
    accounts: HashMap<String, Account>,
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Create a new user account
    pub fn create_account(&mut self, public_key: Option<PublicKey>) -> String {
        let account_id = Uuid::new_v4().to_string();

        let account = Account {
            id: account_id.clone(),
            public_key,
            balance: 0,
            created_at: Utc::now(),
            last_transaction: None,
            is_frozen: false,
        };

        self.accounts.insert(account_id.clone(), account);
        account_id
    }

    /// Get account by ID
    pub fn get_account(&self, account_id: &str) -> Result<&Account, AstorError> {
        self.accounts
            .get(account_id)
            .ok_or_else(|| AstorError::AccountNotFound(account_id.to_string()))
    }

    /// Get mutable account by ID
    fn get_account_mut(&mut self, account_id: &str) -> Result<&mut Account, AstorError> {
        self.accounts
            .get_mut(account_id)
            .ok_or_else(|| AstorError::AccountNotFound(account_id.to_string()))
    }

    /// Credit account with amount
    pub fn credit_account(&mut self, account_id: &str, amount: u64) -> Result<(), AstorError> {
        let account = self.get_account_mut(account_id)?;

        if account.is_frozen {
            return Err(AstorError::Unauthorized("Account is frozen".to_string()));
        }

        account.balance = account.balance.checked_add(amount).ok_or_else(|| {
            AstorError::TransactionValidationFailed("Balance overflow".to_string())
        })?;
        account.last_transaction = Some(Utc::now());

        Ok(())
    }

    /// Debit account with amount
    pub fn debit_account(&mut self, account_id: &str, amount: u64) -> Result<(), AstorError> {
        let account = self.get_account_mut(account_id)?;

        if account.is_frozen {
            return Err(AstorError::Unauthorized("Account is frozen".to_string()));
        }

        if account.balance < amount {
            return Err(AstorError::InsufficientFunds);
        }

        account.balance -= amount;
        account.last_transaction = Some(Utc::now());

        Ok(())
    }

    /// Check if account has sufficient balance
    pub fn has_sufficient_balance(
        &self,
        account_id: &str,
        amount: u64,
    ) -> Result<bool, AstorError> {
        let account = self.get_account(account_id)?;
        Ok(account.balance >= amount)
    }

    /// Verify transfer authorization (signature check)
    pub fn verify_transfer_authorization(
        &self,
        account_id: &str,
        signature: &Signature,
    ) -> Result<(), AstorError> {
        let account = self.get_account(account_id)?;

        if let Some(public_key) = &account.public_key {
            let message = format!("transfer_from_{}", account_id);
            signature.verify(public_key, message.as_bytes())?;
        } else {
            return Err(AstorError::Unauthorized(
                "Account has no public key for verification".to_string(),
            ));
        }

        Ok(())
    }

    /// Freeze/unfreeze account
    pub fn set_account_frozen(&mut self, account_id: &str, frozen: bool) -> Result<(), AstorError> {
        let account = self.get_account_mut(account_id)?;
        account.is_frozen = frozen;
        Ok(())
    }

    /// Get account balance
    pub fn get_balance(&self, account_id: &str) -> Result<u64, AstorError> {
        let account = self.get_account(account_id)?;
        Ok(account.balance)
    }
}
