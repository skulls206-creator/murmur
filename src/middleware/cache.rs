use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use moka::future::Cache;
use sha2::{Digest, Sha256};
use tracing::debug;

/// Cache middleware for feed responses
///
/// Caches HTML responses for non-authenticated users.
/// Cache key is the SHA-256 hash of the request path + query string.
/// TTL is configured via MURMUR_CACHE_TTL (default 300s).
///
/// Authenticated users bypass the cache because their responses
/// contain personalized data (voted state, saved state, etc.).

pub struct CacheLayer {
    cache: Cache<u64, CachedResponse>,
    ttl: Duration,
}

struct CachedResponse {
    status: StatusCode,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl CacheLayer {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(ttl_seconds))
                .build(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn cache_middleware(
        State(state): State<Arc<CacheLayer>>,
        request: Request,
        next: Next,
    ) -> Response {
        // Only cache GET requests
        if request.method() != axum::http::Method::GET {
            return next.run(request).await;
        }

        // Don't cache authenticated requests
        let is_authenticated = request
            .extensions()
            .get::<crate::middleware::auth::AuthenticatedUser>()
            .and_then(|u| u.0.is_some())
            .unwrap_or(false);

        if is_authenticated {
            return next.run(request).await;
        }

        // Build cache key from path + query
        let uri = request.uri();
        let key_input = format!("{}:{}", uri.path(), uri.query().unwrap_or(""));
        let mut hasher = Sha256::new();
        hasher.update(key_input.as_bytes());
        let cache_key = u64::from_be_bytes(
            hasher.finalize()[..8].try_into().unwrap(),
        );

        // Check cache
        if let Some(cached) = state.cache.get(&cache_key).await {
            debug!("Cache hit: {}", uri.path());
            let mut resp = Response::builder().status(cached.status);
            for (key, value) in &cached.headers {
                resp = resp.header(key, value);
            }
            resp = resp.header("x-murmur-cache", "hit");
            return resp.body(Body::from(cached.body.clone())).unwrap();
        }

        // Not cached — run the handler
        let response = next.run(request).await;

        // Only cache successful responses
        if response.status().is_success() {
            let (parts, body) = response.into_parts();
            let body_bytes = axum::body::to_bytes(body, 5 * 1024 * 1024).await.unwrap_or_default();

            let headers: Vec<(String, String)> = parts
                .headers
                .iter()
                .filter(|(k, _)| {
                    // Don't cache set-cookie or vary headers
                    k.as_str() != "set-cookie"
                })
                .map(|(k, v)| {
                    (k.as_str().to_string(), v.to_str().unwrap_or("").to_string())
                })
                .collect();

            let cached = CachedResponse {
                status: parts.status,
                headers,
                body: body_bytes.to_vec(),
            };

            state.cache.insert(cache_key, cached).await;

            let mut resp = Response::from_parts(parts, Body::from(body_bytes));
            resp.headers_mut()
                .insert("x-murmur-cache", "miss".parse().unwrap());
            return resp;
        }

        response
    }
}
