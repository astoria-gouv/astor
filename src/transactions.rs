//! Transaction management and validation module

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::AstorError;

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Issuance {
        issuer: String,
        recipient: String,
        amount: u64,
    },
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
    Conversion {
        account: String,
        from_currency: String,
        to_currency: String,
        amount: u64,
        exchange_rate: f64,
    },
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub hash: String,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed(String),
}

/// Manages transaction creation and validation
pub struct TransactionManager {
    transactions: Vec<Transaction>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }

    /// Create an issuance transaction
    pub fn create_issuance(
        &mut self,
        issuer: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<String, AstorError> {
        let tx_id = Uuid::new_v4().to_string();
        
        let transaction_type = TransactionType::Issuance {
            issuer: issuer.to_string(),
            recipient: recipient.to_string(),
            amount,
        };

        let transaction = Transaction {
            id: tx_id.clone(),
            transaction_type: transaction_type.clone(),
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
            hash: self.calculate_transaction_hash(&tx_id, &transaction_type),
        };

        self.transactions.push(transaction);
        Ok(tx_id)
    }

    /// Create a transfer transaction
    pub fn create_transfer(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<String, AstorError> {
        let tx_id = Uuid::new_v4().to_string();
        
        let transaction_type = TransactionType::Transfer {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        };

        let transaction = Transaction {
            id: tx_id.clone(),
            transaction_type: transaction_type.clone(),
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
            hash: self.calculate_transaction_hash(&tx_id, &transaction_type),
        };

        self.transactions.push(transaction);
        Ok(tx_id)
    }

    /// Confirm a transaction
    pub fn confirm_transaction(&mut self, tx_id: &str) -> Result<(), AstorError> {
        if let Some(tx) = self.transactions.iter_mut().find(|t| t.id == tx_id) {
            tx.status = TransactionStatus::Confirmed;
            Ok(())
        } else {
            Err(AstorError::TransactionValidationFailed("Transaction not found".to_string()))
        }
    }

    /// Fail a transaction
    pub fn fail_transaction(&mut self, tx_id: &str, reason: String) -> Result<(), AstorError> {
        if let Some(tx) = self.transactions.iter_mut().find(|t| t.id == tx_id) {
            tx.status = TransactionStatus::Failed(reason);
            Ok(())
        } else {
            Err(AstorError::TransactionValidationFailed("Transaction not found".to_string()))
        }
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|t| t.id == tx_id)
    }

    /// Get all transactions
    pub fn get_all_transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    /// Calculate transaction hash for integrity
    fn calculate_transaction_hash(&self, tx_id: &str, tx_type: &TransactionType) -> String {
        use crate::security::hash_data;
        let data = format!("{}{:?}", tx_id, tx_type);
        hash_data(data.as_bytes())
    }
}
