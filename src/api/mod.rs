//! REST API layer for the Astor currency system

pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;

use axum::{
    http::{Method, StatusCode},
    response::Json,
    Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::config::Config;
use crate::database::Database;

/// API application state
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub config: Config,
}

/// Create the main API router
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .nest("/api/v1", routes::create_api_routes())
        .route("/health", axum::routing::get(health_check))
        .route("/metrics", axum::routing::get(metrics))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(middleware::timeout::TimeoutLayer::new(Duration::from_secs(
                    30,
                )))
                .layer(middleware::rate_limit::RateLimitLayer::new(
                    100,
                    Duration::from_secs(60),
                )),
        )
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "astor-currency",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now()
    })))
}

/// Metrics endpoint
async fn metrics() -> Result<Json<Value>, StatusCode> {
    // In production, this would integrate with Prometheus or similar
    Ok(Json(json!({
        "uptime": "placeholder",
        "requests_total": 0,
        "active_connections": 0
    })))
}
