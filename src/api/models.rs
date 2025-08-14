//! API request and response models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Authentication models
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub signature: String, // Base64 encoded signature
    pub message: String,   // Message that was signed
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// Account models
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub public_key: Option<String>, // Base64 encoded
    pub account_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_transaction: Option<DateTime<Utc>>,
    pub is_frozen: bool,
    pub account_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub is_frozen: Option<bool>,
}

// Transaction models
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub transaction_type: String,
    pub from_account: Option<Uuid>,
    pub to_account: Uuid,
    pub amount: i64,
    pub signature: Option<String>, // Base64 encoded
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from_account: Uuid,
    pub to_account: Uuid,
    pub amount: i64,
    pub signature: String, // Base64 encoded signature
}

#[derive(Debug, Deserialize)]
pub struct IssueCurrencyRequest {
    pub recipient_account: Uuid,
    pub amount: i64,
    pub admin_signature: String, // Base64 encoded
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub transaction_type: String,
    pub from_account: Option<Uuid>,
    pub to_account: Option<Uuid>,
    pub amount: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

// Admin models
#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub public_key: String, // Base64 encoded
    pub role: String,
    pub permissions: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AdminResponse {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

// Ledger models
#[derive(Debug, Serialize)]
pub struct LedgerEntryResponse {
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

#[derive(Debug, Serialize)]
pub struct SystemStatsResponse {
    pub total_accounts: i64,
    pub total_supply: i64,
    pub total_transactions: i64,
    pub ledger_entries: i64,
    pub active_admins: i64,
}

// Common response models
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// Query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
        }
    }
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}
