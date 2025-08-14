//! Ledger repository for database operations

use crate::database::models::LedgerEntryModel;
use crate::errors::AstorError;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct LedgerRepository {
    pool: PgPool,
}

impl LedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add new ledger entry
    pub async fn add_entry(
        &self,
        entry_type: &str,
        transaction_id: Option<Uuid>,
        from_account: Option<Uuid>,
        to_account: Option<Uuid>,
        amount: Option<i64>,
        metadata: serde_json::Value,
        hash: &str,
        previous_hash: &str,
    ) -> Result<LedgerEntryModel, AstorError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Get next block height
        let block_height: i64 =
            sqlx::query_scalar("SELECT COALESCE(MAX(block_height), 0) + 1 FROM ledger_entries")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AstorError::DatabaseError(format!("Failed to get block height: {}", e))
                })?;

        let entry = sqlx::query_as::<_, LedgerEntryModel>(
            r#"
            INSERT INTO ledger_entries 
            (id, entry_type, transaction_id, from_account, to_account, amount, metadata, hash, previous_hash, timestamp, block_height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(entry_type)
        .bind(transaction_id)
        .bind(from_account)
        .bind(to_account)
        .bind(amount)
        .bind(metadata)
        .bind(hash)
        .bind(previous_hash)
        .bind(now)
        .bind(block_height)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to add ledger entry: {}", e)))?;

        Ok(entry)
    }

    /// Get ledger entries with pagination
    pub async fn get_entries(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LedgerEntryModel>, AstorError> {
        let entries = sqlx::query_as::<_, LedgerEntryModel>(
            r#"
            SELECT * FROM ledger_entries 
            ORDER BY block_height ASC 
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to get ledger entries: {}", e)))?;

        Ok(entries)
    }

    /// Get last ledger entry
    pub async fn get_last_entry(&self) -> Result<Option<LedgerEntryModel>, AstorError> {
        let entry = sqlx::query_as::<_, LedgerEntryModel>(
            "SELECT * FROM ledger_entries ORDER BY block_height DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to get last entry: {}", e)))?;

        Ok(entry)
    }

    /// Verify ledger integrity
    pub async fn verify_integrity(&self) -> Result<bool, AstorError> {
        let entries = sqlx::query_as::<_, LedgerEntryModel>(
            "SELECT * FROM ledger_entries ORDER BY block_height ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AstorError::DatabaseError(format!("Failed to get entries for verification: {}", e))
        })?;

        if entries.is_empty() {
            return Ok(true);
        }

        for (i, entry) in entries.iter().enumerate() {
            let expected_previous_hash = if i == 0 {
                "genesis".to_string()
            } else {
                entries[i - 1].hash.clone()
            };

            if entry.previous_hash != expected_previous_hash {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get total supply from ledger
    pub async fn get_total_supply(&self) -> Result<i64, AstorError> {
        let supply: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount), 0) 
            FROM ledger_entries 
            WHERE entry_type = 'issuance'
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to get total supply: {}", e)))?;

        Ok(supply.unwrap_or(0))
    }
}
