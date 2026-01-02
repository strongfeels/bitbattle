use axum::{
    body::Body,
    http::{header, Request, Response},
    middleware::Next,
};
use std::time::Instant;
use uuid::Uuid;

/// Security headers middleware
pub async fn security_headers(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        "DENY".parse().unwrap(),
    );

    // XSS protection (legacy browsers)
    headers.insert(
        "X-XSS-Protection",
        "1; mode=block".parse().unwrap(),
    );

    // Referrer policy
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Content Security Policy (adjust as needed)
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; frame-ancestors 'none'".parse().unwrap(),
    );

    // Permissions Policy
    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    response
}

/// Request ID middleware - adds unique ID to each request for tracing
pub async fn request_id(
    mut request: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Generate or extract request ID
    let request_id = request
        .headers()
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add to request extensions for handlers to access
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap(),
    );

    response
}

/// Request ID extractor
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Request timing middleware - logs request duration
pub async fn request_timing(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log request with timing
    if duration.as_millis() > 1000 {
        tracing::warn!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Slow request"
        );
    } else {
        tracing::info!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }

    response
}
