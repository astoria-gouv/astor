//! Database layer for persistent storage

pub mod models;
pub mod migrations;
pub mod repositories;

use crate::errors::AstorError;
use sqlx::{PgPool, Row};

/// Database connection pool wrapper
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create new database connection
    pub async fn new(database_url: &str) -> Result<Self, AstorError> {
        let pool = PgPool::connect(database_url).await.map_err(|e| {
            AstorError::DatabaseError(format!("Failed to connect to database: {}", e))
        })?;

        Ok(Self { pool })
    }

    /// Get database pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), AstorError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AstorError::DatabaseError(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<(), AstorError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AstorError::DatabaseError(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}
