//! Account management API handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

use crate::api::{
    models::{
        AccountResponse, ApiResponse, CreateAccountRequest, 
        PaginatedResponse, PaginationQuery, UpdateAccountRequest
    },
    AppState,
};
use crate::database::repositories::AccountRepository;

/// Create a new account
pub async fn create_account(
    State(state): State<AppState>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<Json<ApiResponse<AccountResponse>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());
    
    // Decode public key if provided
    let public_key = if let Some(key_str) = request.public_key {
        Some(base64::decode(&key_str).map_err(|_| StatusCode::BAD_REQUEST)?)
    } else {
        None
    };

    let account_type = request.account_type.unwrap_or_else(|| "user".to_string());

    match repo.create_account(public_key, &account_type).await {
        Ok(account) => {
            let response = AccountResponse {
                id: account.id,
                balance: account.balance,
                created_at: account.created_at,
                updated_at: account.updated_at,
                last_transaction: account.last_transaction,
                is_frozen: account.is_frozen,
                account_type: account.account_type,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            tracing::error!("Failed to create account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get account by ID
pub async fn get_account(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<AccountResponse>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());

    match repo.get_account(account_id).await {
        Ok(account) => {
            let response = AccountResponse {
                id: account.id,
                balance: account.balance,
                created_at: account.created_at,
                updated_at: account.updated_at,
                last_transaction: account.last_transaction,
                is_frozen: account.is_frozen,
                account_type: account.account_type,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// List accounts with pagination
pub async fn list_accounts(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<AccountResponse>>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());
    
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    let accounts_result = repo.list_accounts(per_page, offset).await;
    let total_result = repo.count_accounts().await;

    match (accounts_result, total_result) {
        (Ok(accounts), Ok(total)) => {
            let account_responses: Vec<AccountResponse> = accounts
                .into_iter()
                .map(|account| AccountResponse {
                    id: account.id,
                    balance: account.balance,
                    created_at: account.created_at,
                    updated_at: account.updated_at,
                    last_transaction: account.last_transaction,
                    is_frozen: account.is_frozen,
                    account_type: account.account_type,
                })
                .collect();

            let total_pages = (total + per_page - 1) / per_page;
            let response = PaginatedResponse {
                data: account_responses,
                total,
                page,
                per_page,
                total_pages,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get account balance
pub async fn get_balance(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<i64>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());

    match repo.get_account(account_id).await {
        Ok(account) => Ok(Json(ApiResponse::success(account.balance))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Update account
pub async fn update_account(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());

    if let Some(frozen) = request.is_frozen {
        match repo.set_frozen(account_id, frozen).await {
            Ok(_) => Ok(Json(ApiResponse::success(()))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// Freeze account
pub async fn freeze_account(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());

    match repo.set_frozen(account_id, true).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Unfreeze account
pub async fn unfreeze_account(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let repo = AccountRepository::new(state.database.pool().clone());

    match repo.set_frozen(account_id, false).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get account transactions
pub async fn get_account_transactions(
    State(_state): State<AppState>,
    Path(_account_id): Path<Uuid>,
    Query(_pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>, StatusCode> {
    // TODO: Implement transaction history retrieval
    Err(StatusCode::NOT_IMPLEMENTED)
}
