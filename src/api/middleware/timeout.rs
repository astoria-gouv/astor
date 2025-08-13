//! Request timeout middleware

use axum::{extract::Request, http::StatusCode, response::Response};
use std::time::Duration;
use tower::{Layer, Service, ServiceExt};

#[derive(Clone)]
pub struct TimeoutLayer {
    timeout: Duration,
}

impl TimeoutLayer {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = tower::timeout::Timeout<S>;

    fn layer(&self, service: S) -> Self::Service {
        tower::timeout::Timeout::new(service, self.timeout)
    }
}
