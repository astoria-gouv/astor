//! Database models and schemas

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

/// Database model for accounts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountModel {
    pub id: Uuid,
    pub public_key: Option<Vec<u8>>,
    pub balance: i64, // Using i64 for PostgreSQL compatibility
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_transaction: Option<DateTime<Utc>>,
    pub is_frozen: bool,
    pub account_type: String,
}

/// Database model for ledger entries
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LedgerEntryModel {
    pub id: Uuid,
    pub entry_type: String,
    pub transaction_id: Option<Uuid>,
    pub from_account: Option<Uuid>,
    pub to_account: Option<Uuid>,
    pub amount: Option<i64>,
    pub metadata: serde_json::Value,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: DateTime<Utc>,
    pub block_height: i64,
}

/// Database model for transactions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionModel {
    pub id: Uuid,
    pub transaction_type: String,
    pub from_account: Option<Uuid>,
    pub to_account: Option<Uuid>,
    pub amount: i64,
    pub status: String,
    pub signature: Option<Vec<u8>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Database model for administrators
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminModel {
    pub id: Uuid,
    pub username: String,
    pub public_key: Vec<u8>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub permissions: serde_json::Value,
}

/// Database model for audit logs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogModel {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub admin_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Database model for system configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigModel {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

/// Database model for currency conversion records
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversionRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    pub original_amount: i64,
    pub converted_amount: i64,
    pub exchange_rate: f64,
    pub fees: i64,
    pub slippage: f64,
    pub provider: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Database model for exchange rates
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExchangeRateModel {
    pub id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub bid: f64,
    pub ask: f64,
    pub volatility: f64,
    pub daily_change: f64,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
