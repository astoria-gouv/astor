use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = Uuid::new_v4().to_string();

    // Log incoming request
    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Incoming request"
    );

    // Process request
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log response
    if status.is_success() {
        info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed successfully"
        );
    } else if status.is_client_error() {
        warn!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        error!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed with server error"
        );
    }

    response
}

pub async fn security_logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Log security-relevant information
    if let Some(user_agent) = headers.get("user-agent") {
        info!(
            method = %method,
            uri = %uri,
            user_agent = ?user_agent,
            "Security log: User agent"
        );
    }

    if let Some(auth_header) = headers.get("authorization") {
        info!(
            method = %method,
            uri = %uri,
            "Security log: Authentication attempt"
        );
    }

    next.run(request).await
}
