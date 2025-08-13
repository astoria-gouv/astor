//! Rate limiting middleware

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RateLimitLayer {
    max_requests: u32,
    window: Duration,
    store: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            max_requests: self.max_requests,
            window: self.window,
            store: self.store.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    max_requests: u32,
    window: Duration,
    store: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl<S> Service<Request> for RateLimitService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let client_ip = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|hv| hv.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let mut store = self.store.lock().unwrap();
        let now = Instant::now();

        // Clean up expired entries
        store.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.window);

        let (count, _) = store.entry(client_ip.clone()).or_insert((0, now));
        
        if *count >= self.max_requests {
            drop(store);
            return Box::pin(async move {
                Ok(Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .body(axum::body::Body::empty())
                    .unwrap())
            });
        }

        *count += 1;
        drop(store);

        let mut inner = self.inner.clone();
        Box::pin(async move { inner.call(request).await })
    }
}
