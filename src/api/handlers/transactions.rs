use axum::{
    extract::{Json, Path, Query, State},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use crate::{
    errors::AstorError,
    transactions::{Transaction, TransactionManager, TransactionType},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub from_account: String,
    pub to_account: String,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub account_id: Option<String>,
    pub transaction_type: Option<TransactionType>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub transactions: Vec<Transaction>,
    pub total_count: usize,
}

pub async fn create_transaction(
    State(state): State<AppState>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<ResponseJson<Transaction>, AstorError> {
    let mut transaction_manager = TransactionManager::new();
    
    let transaction = transaction_manager.create_transaction(
        request.from_account,
        request.to_account,
        request.amount,
        request.transaction_type,
        request.description,
    )?;
    
    Ok(ResponseJson(transaction))
}

pub async fn get_transactions(
    State(state): State<AppState>,
    Query(query): Query<TransactionQuery>,
) -> Result<ResponseJson<TransactionResponse>, AstorError> {
    let transaction_manager = TransactionManager::new();
    
    let transactions = transaction_manager.get_transactions(
        query.account_id.as_deref(),
        query.transaction_type,
        query.limit.unwrap_or(100),
        query.offset.unwrap_or(0),
    )?;
    
    Ok(ResponseJson(TransactionResponse {
        total_count: transactions.len(),
        transactions,
    }))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(transaction_id): Path<String>,
) -> Result<ResponseJson<Transaction>, AstorError> {
    let transaction_manager = TransactionManager::new();
    
    let transaction = transaction_manager.get_transaction(&transaction_id)?
        .ok_or(AstorError::NotFound("Transaction not found".to_string()))?;
    
    Ok(ResponseJson(transaction))
}

pub async fn cancel_transaction(
    State(state): State<AppState>,
    Path(transaction_id): Path<String>,
) -> Result<ResponseJson<Transaction>, AstorError> {
    let mut transaction_manager = TransactionManager::new();
    
    let transaction = transaction_manager.cancel_transaction(&transaction_id)?;
    
    Ok(ResponseJson(transaction))
}
