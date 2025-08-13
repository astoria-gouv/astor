//! Secure ledger system for recording all transactions

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::security::hash_data;
use crate::errors::AstorError;

/// Ledger entry for recording transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub entry_type: LedgerEntryType,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
    pub previous_hash: String,
}

/// Types of ledger entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerEntryType {
    Issuance {
        transaction_id: String,
        issuer: String,
        recipient: String,
        amount: u64,
    },
    Transfer {
        transaction_id: String,
        from: String,
        to: String,
        amount: u64,
    },
    AccountCreation {
        account_id: String,
    },
    AdminAction {
        admin_id: String,
        action: String,
        target: String,
    },
}

/// Secure, tamper-evident ledger
pub struct Ledger {
    entries: Vec<LedgerEntry>,
    account_balances: HashMap<String, u64>,
    total_supply: u64,
}

impl Ledger {
    /// Create a new ledger
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            account_balances: HashMap::new(),
            total_supply: 0,
        }
    }

    /// Record currency issuance
    pub fn record_issuance(
        &mut self,
        transaction_id: String,
        issuer: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(), AstorError> {
        let entry_type = LedgerEntryType::Issuance {
            transaction_id,
            issuer: issuer.to_string(),
            recipient: recipient.to_string(),
            amount,
        };

        self.add_entry(entry_type)?;
        
        // Update total supply
        self.total_supply = self.total_supply.checked_add(amount)
            .ok_or_else(|| AstorError::LedgerError("Total supply overflow".to_string()))?;

        // Update recipient balance
        let balance = self.account_balances.entry(recipient.to_string()).or_insert(0);
        *balance = balance.checked_add(amount)
            .ok_or_else(|| AstorError::LedgerError("Account balance overflow".to_string()))?;

        Ok(())
    }

    /// Record transfer between accounts
    pub fn record_transfer(
        &mut self,
        transaction_id: String,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<(), AstorError> {
        let entry_type = LedgerEntryType::Transfer {
            transaction_id,
            from: from.to_string(),
            to: to.to_string(),
            amount,
        };

        self.add_entry(entry_type)?;

        // Update balances
        let from_balance = self.account_balances.entry(from.to_string()).or_insert(0);
        if *from_balance < amount {
            return Err(AstorError::LedgerError("Insufficient balance in ledger".to_string()));
        }
        *from_balance -= amount;

        let to_balance = self.account_balances.entry(to.to_string()).or_insert(0);
        *to_balance = to_balance.checked_add(amount)
            .ok_or_else(|| AstorError::LedgerError("Account balance overflow".to_string()))?;

        Ok(())
    }

    /// Record account creation
    pub fn record_account_creation(&mut self, account_id: String) -> Result<(), AstorError> {
        let entry_type = LedgerEntryType::AccountCreation { account_id };
        self.add_entry(entry_type)
    }

    /// Record admin action
    pub fn record_admin_action(
        &mut self,
        admin_id: String,
        action: String,
        target: String,
    ) -> Result<(), AstorError> {
        let entry_type = LedgerEntryType::AdminAction {
            admin_id,
            action,
            target,
        };
        self.add_entry(entry_type)
    }

    /// Add a new entry to the ledger
    fn add_entry(&mut self, entry_type: LedgerEntryType) -> Result<(), AstorError> {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let previous_hash = self.get_last_hash();
        
        // Calculate hash for this entry
        let entry_data = format!("{}{:?}{}", entry_id, entry_type, timestamp);
        let hash = hash_data(format!("{}{}", previous_hash, entry_data).as_bytes());

        let entry = LedgerEntry {
            id: entry_id,
            entry_type,
            timestamp,
            hash,
            previous_hash,
        };

        self.entries.push(entry);
        Ok(())
    }

    /// Get the hash of the last entry (for chaining)
    fn get_last_hash(&self) -> String {
        self.entries
            .last()
            .map(|entry| entry.hash.clone())
            .unwrap_or_else(|| "genesis".to_string())
    }

    /// Verify ledger integrity
    pub fn verify_integrity(&self) -> Result<bool, AstorError> {
        if self.entries.is_empty() {
            return Ok(true);
        }

        for (i, entry) in self.entries.iter().enumerate() {
            let expected_previous_hash = if i == 0 {
                "genesis".to_string()
            } else {
                self.entries[i - 1].hash.clone()
            };

            if entry.previous_hash != expected_previous_hash {
                return Ok(false);
            }

            // Verify entry hash
            let entry_data = format!("{}{:?}{}", entry.id, entry.entry_type, entry.timestamp);
            let expected_hash = hash_data(format!("{}{}", entry.previous_hash, entry_data).as_bytes());
            
            if entry.hash != expected_hash {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get all ledger entries
    pub fn get_entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Get total supply
    pub fn get_total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Get account balance from ledger
    pub fn get_account_balance(&self, account_id: &str) -> u64 {
        self.account_balances.get(account_id).copied().unwrap_or(0)
    }
}
