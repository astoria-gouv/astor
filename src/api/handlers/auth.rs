use crate::{
    errors::AstorError,
    security::{AuthenticationManager, SessionManager},
    AppState,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: i64,
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<ResponseJson<LoginResponse>, AstorError> {
    let auth_manager = AuthenticationManager::new();

    // Authenticate user
    let user_id = auth_manager
        .authenticate_user(
            &request.username,
            &request.password,
            request.totp_code.as_deref(),
        )
        .await?;

    // Create session
    let session_manager = SessionManager::new("your-secret-key".to_string());
    let session = session_manager
        .create_session(&user_id, vec!["user".to_string()])
        .await?;

    Ok(ResponseJson(LoginResponse {
        token: session.token,
        expires_at: session.expires_at,
        user_id,
    }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<ResponseJson<LoginResponse>, AstorError> {
    let session_manager = SessionManager::new("your-secret-key".to_string());

    let session = session_manager
        .refresh_session(&request.refresh_token)
        .await?;

    Ok(ResponseJson(LoginResponse {
        token: session.token,
        expires_at: session.expires_at,
        user_id: session.user_id,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(token): Json<String>,
) -> Result<StatusCode, AstorError> {
    let session_manager = SessionManager::new("your-secret-key".to_string());
    session_manager.invalidate_session(&token).await?;

    Ok(StatusCode::OK)
}
