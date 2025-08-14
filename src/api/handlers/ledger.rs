use crate::{
    errors::AstorError,
    ledger::{Ledger, LedgerEntry},
    AppState,
};
use axum::{
    extract::{Json, Path, Query, State},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LedgerQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub account_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LedgerResponse {
    pub entries: Vec<LedgerEntry>,
    pub total_count: usize,
}

pub async fn get_ledger_entries(
    State(state): State<AppState>,
    Query(query): Query<LedgerQuery>,
) -> Result<ResponseJson<LedgerResponse>, AstorError> {
    let ledger = Ledger::new();

    let entries = if let Some(account_id) = query.account_id {
        ledger.get_account_history(&account_id)?
    } else {
        ledger.get_all_entries()?
    };

    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    let paginated_entries: Vec<LedgerEntry> =
        entries.into_iter().skip(offset).take(limit).collect();

    Ok(ResponseJson(LedgerResponse {
        total_count: paginated_entries.len(),
        entries: paginated_entries,
    }))
}

pub async fn get_ledger_entry(
    State(state): State<AppState>,
    Path(entry_id): Path<String>,
) -> Result<ResponseJson<LedgerEntry>, AstorError> {
    let ledger = Ledger::new();

    let entry = ledger
        .get_entry(&entry_id)?
        .ok_or(AstorError::NotFound("Ledger entry not found".to_string()))?;

    Ok(ResponseJson(entry))
}

pub async fn verify_ledger_integrity(
    State(state): State<AppState>,
) -> Result<ResponseJson<bool>, AstorError> {
    let ledger = Ledger::new();
    let is_valid = ledger.verify_integrity()?;

    Ok(ResponseJson(is_valid))
}
