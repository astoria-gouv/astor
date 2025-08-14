use crate::database::models::AdminRecord;
use crate::errors::AstorError;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_admin(&self, admin: &AdminRecord) -> Result<(), AstorError> {
        sqlx::query!(
            r#"
            INSERT INTO admins (id, username, email, role, permissions, password_hash, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            admin.id,
            admin.username,
            admin.email,
            admin.role,
            &admin.permissions,
            admin.password_hash,
            admin.is_active,
            admin.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_admin(&self, id: Uuid) -> Result<Option<AdminRecord>, AstorError> {
        let row = sqlx::query!("SELECT * FROM admins WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(AdminRecord {
                id: row.id,
                username: row.username,
                email: row.email,
                role: row.role,
                permissions: row.permissions,
                password_hash: row.password_hash,
                is_active: row.is_active,
                last_login: row.last_login,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_admin_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AdminRecord>, AstorError> {
        let row = sqlx::query!("SELECT * FROM admins WHERE username = $1", username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(AdminRecord {
                id: row.id,
                username: row.username,
                email: row.email,
                role: row.role,
                permissions: row.permissions,
                password_hash: row.password_hash,
                is_active: row.is_active,
                last_login: row.last_login,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_admins(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AdminRecord>, AstorError> {
        let rows = sqlx::query!(
            "SELECT * FROM admins ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        let admins = rows
            .into_iter()
            .map(|row| AdminRecord {
                id: row.id,
                username: row.username,
                email: row.email,
                role: row.role,
                permissions: row.permissions,
                password_hash: row.password_hash,
                is_active: row.is_active,
                last_login: row.last_login,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(admins)
    }

    pub async fn update_last_login(&self, id: Uuid) -> Result<(), AstorError> {
        sqlx::query!(
            "UPDATE admins SET last_login = NOW(), updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn deactivate_admin(&self, id: Uuid) -> Result<(), AstorError> {
        sqlx::query!(
            "UPDATE admins SET is_active = false, updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
