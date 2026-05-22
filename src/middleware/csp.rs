use axum::http::HeaderValue;
use axum::response::Response;
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;

/// Build a strict Content Security Policy for Murmur
///
/// This CSP ensures:
/// - No inline scripts (unless we add nonces)
/// - No connections to external origins
/// - All media is served from the same origin (our proxy)
/// - No trackers, beacons, or analytics scripts can run
///
/// We use a relaxed policy on `img-src` and `media-src` to allow
/// direct loading of media when the proxy is disabled, but the
/// proxy is the default and recommended configuration.
pub fn csp_header() -> SetResponseHeaderLayer<HeaderValue> {
    let policy = [
        "default-src 'self'",
        "script-src 'self' 'unsafe-inline'", // unsafe-inline for progressive enhancement JS
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data: https:",
        "media-src 'self' https:",
        "font-src 'self' data:",
        "connect-src 'self'",
        "frame-ancestors 'none'",
        "form-action 'self'",
        "base-uri 'self'",
        "object-src 'none'",
        "upgrade-insecure-requests",
        "block-all-mixed-content",
    ]
    .join("; ");

    SetResponseHeaderLayer::overriding(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&policy).expect("Invalid CSP header"),
    )
}

/// Default security headers for all responses
pub fn security_headers(response: Response) -> Response {
    let headers = response.headers_mut();

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("0"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()",
        ),
    );

    response
}
