use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::errors::AstorError;
use crate::database::models::TransactionRecord;

#[derive(Clone)]
pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_transaction(&self, transaction: &TransactionRecord) -> Result<(), AstorError> {
        sqlx::query!(
            r#"
            INSERT INTO transactions (id, from_account, to_account, amount, currency, transaction_type, status, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            transaction.id,
            transaction.from_account,
            transaction.to_account,
            transaction.amount,
            transaction.currency,
            transaction.transaction_type,
            transaction.status,
            transaction.metadata,
            transaction.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_transaction(&self, id: Uuid) -> Result<Option<TransactionRecord>, AstorError> {
        let row = sqlx::query!(
            "SELECT * FROM transactions WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(TransactionRecord {
                id: row.id,
                from_account: row.from_account,
                to_account: row.to_account,
                amount: row.amount,
                currency: row.currency,
                transaction_type: row.transaction_type,
                status: row.status,
                metadata: row.metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_transactions_by_account(&self, account_id: Uuid, limit: i64, offset: i64) -> Result<Vec<TransactionRecord>, AstorError> {
        let rows = sqlx::query!(
            r#"
            SELECT * FROM transactions 
            WHERE from_account = $1 OR to_account = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            account_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        let transactions = rows.into_iter().map(|row| TransactionRecord {
            id: row.id,
            from_account: row.from_account,
            to_account: row.to_account,
            amount: row.amount,
            currency: row.currency,
            transaction_type: row.transaction_type,
            status: row.status,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect();

        Ok(transactions)
    }

    pub async fn update_transaction_status(&self, id: Uuid, status: String) -> Result<(), AstorError> {
        sqlx::query!(
            "UPDATE transactions SET status = $1, updated_at = NOW() WHERE id = $2",
            status,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_transaction_volume(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Decimal, AstorError> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_volume
            FROM transactions 
            WHERE created_at BETWEEN $1 AND $2 AND status = 'completed'
            "#,
            start_date,
            end_date
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(row.total_volume.unwrap_or_default())
    }
}
