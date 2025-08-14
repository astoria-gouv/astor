use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::errors::AstorError;
use crate::database::models::AuditRecord;

#[derive(Clone)]
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_audit_log(&self, audit: &AuditRecord) -> Result<(), AstorError> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, old_values, new_values, ip_address, user_agent, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            audit.id,
            audit.user_id,
            audit.action,
            audit.resource_type,
            audit.resource_id,
            audit.old_values,
            audit.new_values,
            audit.ip_address,
            audit.user_agent,
            audit.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_audit_logs(&self, limit: i64, offset: i64) -> Result<Vec<AuditRecord>, AstorError> {
        let rows = sqlx::query!(
            "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        let audit_logs = rows.into_iter().map(|row| AuditRecord {
            id: row.id,
            user_id: row.user_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            old_values: row.old_values,
            new_values: row.new_values,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: row.created_at,
        }).collect();

        Ok(audit_logs)
    }

    pub async fn get_audit_logs_by_user(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<AuditRecord>, AstorError> {
        let rows = sqlx::query!(
            "SELECT * FROM audit_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        let audit_logs = rows.into_iter().map(|row| AuditRecord {
            id: row.id,
            user_id: row.user_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            old_values: row.old_values,
            new_values: row.new_values,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: row.created_at,
        }).collect();

        Ok(audit_logs)
    }

    pub async fn get_audit_logs_by_resource(&self, resource_type: &str, resource_id: Uuid, limit: i64, offset: i64) -> Result<Vec<AuditRecord>, AstorError> {
        let rows = sqlx::query!(
            "SELECT * FROM audit_logs WHERE resource_type = $1 AND resource_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            resource_type,
            resource_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        let audit_logs = rows.into_iter().map(|row| AuditRecord {
            id: row.id,
            user_id: row.user_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            old_values: row.old_values,
            new_values: row.new_values,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: row.created_at,
        }).collect();

        Ok(audit_logs)
    }

    pub async fn get_audit_logs_by_action(&self, action: &str, limit: i64, offset: i64) -> Result<Vec<AuditRecord>, AstorError> {
        let rows = sqlx::query!(
            "SELECT * FROM audit_logs WHERE action = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            action,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AstorError::DatabaseError(e.to_string()))?;

        let audit_logs = rows.into_iter().map(|row| AuditRecord {
            id: row.id,
            user_id: row.user_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            old_values: row.old_values,
            new_values: row.new_values,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: row.created_at,
        }).collect();

        Ok(audit_logs)
    }
}
