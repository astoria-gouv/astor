//! API route definitions

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::{handlers, AppState};

/// Create all API routes
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/accounts", account_routes())
        .nest("/transactions", transaction_routes())
        .nest("/admin", admin_routes())
        .nest("/ledger", ledger_routes())
}

/// Authentication routes
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh_token))
        .route("/logout", post(handlers::auth::logout))
}

/// Account management routes
fn account_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::accounts::create_account))
        .route("/", get(handlers::accounts::list_accounts))
        .route("/:id", get(handlers::accounts::get_account))
        .route("/:id", put(handlers::accounts::update_account))
        .route("/:id/balance", get(handlers::accounts::get_balance))
        .route("/:id/freeze", put(handlers::accounts::freeze_account))
        .route("/:id/unfreeze", put(handlers::accounts::unfreeze_account))
        .route(
            "/:id/transactions",
            get(handlers::accounts::get_account_transactions),
        )
}

/// Transaction routes
fn transaction_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::transactions::create_transaction))
        .route("/", get(handlers::transactions::list_transactions))
        .route("/:id", get(handlers::transactions::get_transaction))
        .route(
            "/:id/status",
            put(handlers::transactions::update_transaction_status),
        )
        .route("/transfer", post(handlers::transactions::transfer))
        .route("/issue", post(handlers::transactions::issue_currency))
}

/// Admin routes
fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::admin::create_admin))
        .route("/", get(handlers::admin::list_admins))
        .route("/:id", get(handlers::admin::get_admin))
        .route("/:id", put(handlers::admin::update_admin))
        .route("/:id/deactivate", put(handlers::admin::deactivate_admin))
        .route("/system/stats", get(handlers::admin::system_stats))
        .route("/audit", get(handlers::admin::audit_logs))
}

/// Ledger query routes
fn ledger_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::ledger::get_ledger_entries))
        .route("/verify", get(handlers::ledger::verify_integrity))
        .route("/supply", get(handlers::ledger::total_supply))
        .route("/stats", get(handlers::ledger::ledger_stats))
}
