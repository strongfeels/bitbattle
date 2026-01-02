use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    net::SocketAddr,
    num::NonZeroU32,
    sync::Arc,
    time::Duration,
};
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

/// Rate limiter configuration for different endpoint types
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Requests per second for general API endpoints
    pub general_rps: u32,
    /// Requests per second for authentication endpoints (stricter)
    pub auth_rps: u32,
    /// Requests per second for code submission (very strict)
    pub submit_rps: u32,
    /// Requests per second for matchmaking
    pub matchmaking_rps: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            general_rps: 100,      // 100 requests per second
            auth_rps: 5,           // 5 auth attempts per second
            submit_rps: 2,         // 2 submissions per second
            matchmaking_rps: 10,   // 10 matchmaking requests per second
        }
    }
}

/// Simple IP-based rate limiter using governor
pub struct IpRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl IpRateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap_or(NonZeroU32::new(1).unwrap()))
            .allow_burst(NonZeroU32::new(requests_per_second * 2).unwrap_or(NonZeroU32::new(1).unwrap()));

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

impl Clone for IpRateLimiter {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
        }
    }
}

/// Layer for applying rate limiting to routes
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: IpRateLimiter,
}

impl RateLimitLayer {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            limiter: IpRateLimiter::new(requests_per_second),
        }
    }

    pub fn general() -> Self {
        Self::new(RateLimitConfig::default().general_rps)
    }

    pub fn auth() -> Self {
        Self::new(RateLimitConfig::default().auth_rps)
    }

    pub fn submit() -> Self {
        Self::new(RateLimitConfig::default().submit_rps)
    }

    pub fn matchmaking() -> Self {
        Self::new(RateLimitConfig::default().matchmaking_rps)
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Service that applies rate limiting
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: IpRateLimiter,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let limiter = self.limiter.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if !limiter.check() {
                // Rate limit exceeded
                let response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("Retry-After", "1")],
                    "Rate limit exceeded. Please slow down.",
                );

                let (parts, _) = Response::new(Body::empty()).into_parts();
                let body = Body::from("Rate limit exceeded. Please slow down.");
                let mut response = Response::from_parts(parts, body);
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                response.headers_mut().insert("Retry-After", "1".parse().unwrap());

                return Ok(response);
            }

            inner.call(req).await
        })
    }
}

/// Response type for rate limit errors
#[derive(serde::Serialize)]
pub struct RateLimitError {
    pub error: String,
    pub retry_after_seconds: u32,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| "Rate limit exceeded".to_string());

        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("Content-Type", "application/json"),
                ("Retry-After", "1"),
            ],
            body,
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_requests_under_limit() {
        let limiter = IpRateLimiter::new(10);

        // Should allow first few requests
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();

        assert_eq!(config.general_rps, 100);
        assert_eq!(config.auth_rps, 5);
        assert_eq!(config.submit_rps, 2);
        assert_eq!(config.matchmaking_rps, 10);
    }
}
