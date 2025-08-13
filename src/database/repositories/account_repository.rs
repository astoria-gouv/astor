//! Account repository for database operations

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::database::models::AccountModel;
use crate::errors::AstorError;

pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new account
    pub async fn create_account(
        &self,
        public_key: Option<Vec<u8>>,
        account_type: &str,
    ) -> Result<AccountModel, AstorError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let account = sqlx::query_as::<_, AccountModel>(
            r#"
            INSERT INTO accounts (id, public_key, balance, created_at, updated_at, is_frozen, account_type)
            VALUES ($1, $2, 0, $3, $3, false, $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(public_key)
        .bind(now)
        .bind(account_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to create account: {}", e)))?;

        Ok(account)
    }

    /// Get account by ID
    pub async fn get_account(&self, account_id: Uuid) -> Result<AccountModel, AstorError> {
        let account = sqlx::query_as::<_, AccountModel>(
            "SELECT * FROM accounts WHERE id = $1"
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AstorError::AccountNotFound(account_id.to_string()),
            _ => AstorError::DatabaseError(format!("Failed to get account: {}", e)),
        })?;

        Ok(account)
    }

    /// Update account balance
    pub async fn update_balance(
        &self,
        account_id: Uuid,
        new_balance: i64,
    ) -> Result<(), AstorError> {
        let now = Utc::now();
        
        let result = sqlx::query(
            r#"
            UPDATE accounts 
            SET balance = $1, updated_at = $2, last_transaction = $2
            WHERE id = $3 AND NOT is_frozen
            "#,
        )
        .bind(new_balance)
        .bind(now)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to update balance: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AstorError::AccountNotFound(account_id.to_string()));
        }

        Ok(())
    }

    /// Freeze/unfreeze account
    pub async fn set_frozen(&self, account_id: Uuid, frozen: bool) -> Result<(), AstorError> {
        let result = sqlx::query(
            "UPDATE accounts SET is_frozen = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(frozen)
        .bind(Utc::now())
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to update account status: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AstorError::AccountNotFound(account_id.to_string()));
        }

        Ok(())
    }

    /// Get accounts with pagination
    pub async fn list_accounts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AccountModel>, AstorError> {
        let accounts = sqlx::query_as::<_, AccountModel>(
            r#"
            SELECT * FROM accounts 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(format!("Failed to list accounts: {}", e)))?;

        Ok(accounts)
    }

    /// Get total account count
    pub async fn count_accounts(&self) -> Result<i64, AstorError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AstorError::DatabaseError(format!("Failed to count accounts: {}", e)))?;

        Ok(count.0)
    }
}
