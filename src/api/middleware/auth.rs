//! Authentication middleware

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,      // Subject (user ID)
    pub role: String,   // User role
    pub exp: i64,       // Expiration time
    pub iat: i64,       // Issued at
}

/// JWT authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];
    
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.security.jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    // Add claims to request extensions for use in handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Admin-only middleware
pub async fn admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !["admin", "root"].contains(&claims.role.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
