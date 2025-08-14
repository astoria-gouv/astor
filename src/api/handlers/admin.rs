//! Admin API handlers for central bank and system administration

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    admin::{AdminManager, Administrator},
    api::{models::*, AppState},
    central_bank::CentralBank,
    errors::AstorError,
    security::{Role, Signature},
};

#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub admin_id: String,
    pub public_key: String, // Base64 encoded Ed25519 public key
    pub role: Role,
}

#[derive(Debug, Serialize)]
pub struct AdminResponse {
    pub id: String,
    pub role: Role,
    pub created_at: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAdminRequest {
    pub is_active: Option<bool>,
    pub role: Option<Role>,
}

#[derive(Debug, Serialize)]
pub struct SystemStatsResponse {
    pub total_admins: usize,
    pub active_admins: usize,
    pub total_money_supply: u64,
    pub base_interest_rate: f64,
    pub registered_banks: usize,
    pub active_banks: usize,
    pub system_status: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub admin_id: Option<String>,
    pub action_type: Option<String>,
}

/// Create a new administrator
pub async fn create_admin(
    State(state): State<AppState>,
    Json(request): Json<CreateAdminRequest>,
) -> Result<Json<AdminResponse>, (StatusCode, Json<ErrorResponse>)> {
    let public_key = ed25519_dalek::PublicKey::from_bytes(
        &base64::decode(&request.public_key).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid public key format".to_string(),
                    message: "Public key must be base64 encoded".to_string(),
                }),
            )
        })?,
    )
    .map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid public key".to_string(),
                message: "Invalid Ed25519 public key".to_string(),
            }),
        )
    })?;

    let mut admin_manager = state.admin_manager.lock().await;
    admin_manager
        .add_admin(request.admin_id.clone(), public_key)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Failed to create admin".to_string(),
                    message: e.to_string(),
                }),
            )
        })?;

    let admin = admin_manager.get_admin(&request.admin_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve created admin".to_string(),
                message: e.to_string(),
            }),
        )
    })?;

    Ok(Json(AdminResponse {
        id: admin.id.clone(),
        role: admin.role.clone(),
        created_at: admin.created_at.to_rfc3339(),
        is_active: admin.is_active,
    }))
}

/// List all administrators
pub async fn list_admins(
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let admin_manager = state.admin_manager.lock().await;
    let admins = admin_manager.list_active_admins();

    let admin_responses: Vec<AdminResponse> = admins
        .into_iter()
        .map(|admin| AdminResponse {
            id: admin.id.clone(),
            role: admin.role.clone(),
            created_at: admin.created_at.to_rfc3339(),
            is_active: admin.is_active,
        })
        .collect();

    Ok(Json(admin_responses))
}

/// Get administrator by ID
pub async fn get_admin(
    State(state): State<AppState>,
    Path(admin_id): Path<String>,
) -> Result<Json<AdminResponse>, (StatusCode, Json<ErrorResponse>)> {
    let admin_manager = state.admin_manager.lock().await;
    let admin = admin_manager.get_admin(&admin_id).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Admin not found".to_string(),
                message: e.to_string(),
            }),
        )
    })?;

    Ok(Json(AdminResponse {
        id: admin.id.clone(),
        role: admin.role.clone(),
        created_at: admin.created_at.to_rfc3339(),
        is_active: admin.is_active,
    }))
}

/// Update administrator
pub async fn update_admin(
    State(state): State<AppState>,
    Path(admin_id): Path<String>,
    Json(request): Json<UpdateAdminRequest>,
) -> Result<Json<AdminResponse>, (StatusCode, Json<ErrorResponse>)> {
    // For now, return method not implemented
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Update admin not implemented".to_string(),
            message: "Admin update functionality needs to be implemented in AdminManager"
                .to_string(),
        }),
    ))
}

/// Deactivate administrator
pub async fn deactivate_admin(
    State(state): State<AppState>,
    Path(admin_id): Path<String>,
) -> Result<Json<AdminResponse>, (StatusCode, Json<ErrorResponse>)> {
    // For now, return method not implemented
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "Deactivate admin not implemented".to_string(),
            message: "Admin deactivation functionality needs to be implemented in AdminManager"
                .to_string(),
        }),
    ))
}

/// Get system statistics
pub async fn system_stats(
    State(state): State<AppState>,
) -> Result<Json<SystemStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let admin_manager = state.admin_manager.lock().await;
    let central_bank = state.central_bank.lock().await;
    let banking_network = state.banking_network.lock().await;

    let active_admins = admin_manager.list_active_admins();
    let money_stats = central_bank.get_money_supply_stats();
    let network_stats = banking_network.get_network_stats().await;

    Ok(Json(SystemStatsResponse {
        total_admins: active_admins.len(),
        active_admins: active_admins.len(),
        total_money_supply: money_stats.total_supply,
        base_interest_rate: money_stats.base_interest_rate,
        registered_banks: network_stats.total_registered_banks,
        active_banks: network_stats.active_banks,
        system_status: "Operational".to_string(),
    }))
}

/// Get audit logs
pub async fn audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, (StatusCode, Json<ErrorResponse>)> {
    // For now, return empty audit logs
    Ok(Json(vec![]))
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub admin_id: String,
    pub action: String,
    pub timestamp: String,
    pub details: HashMap<String, String>,
}
