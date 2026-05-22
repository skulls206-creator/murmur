use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use moka::future::Cache;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::config::Config;
use crate::error::AppError;

/// Query params for media proxy — allows resizing
#[derive(Debug, Deserialize)]
pub struct MediaQuery {
    width: Option<u32>,
    height: Option<u32>,
}

/// Shared state for the media proxy
pub struct MediaProxyState {
    client: Client,
    config: Arc<Config>,
    cache: Cache<String, CachedMedia>,
}

#[derive(Clone)]
struct CachedMedia {
    data: Bytes,
    content_type: String,
    cache_headers: HeaderMap,
}

impl MediaProxyState {
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .gzip(true)
            .brotli(true)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build media proxy client");

        let cache: Cache<String, CachedMedia> = Cache::builder()
            .max_capacity(512)
            .time_to_live(std::time::Duration::from_secs(config.cache_ttl_seconds))
            .build();

        Self {
            client,
            config,
            cache,
        }
    }
}

/// Proxy media (images, gifs, videos) through the server
///
/// Takes an encoded target URL and fetches it server-side,
/// streaming the response back to the user. This ensures:
/// - No direct connections to Reddit's CDN
/// - No tracking pixels or analytics from media hosts
/// - Users' IPs are never exposed to third-party hosts
pub async fn proxy_media(
    State(state): State<Arc<MediaProxyState>>,
    Path(encoded_url): Path<String>,
    Query(_query): Query<MediaQuery>,
) -> Result<Response, AppError> {
    if !state.config.proxy_media {
        return Err(AppError::BadRequest("Media proxy is disabled".into()));
    }

    // Decode the base64-encoded URL
    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &encoded_url,
    )
    .map_err(|_| AppError::BadRequest("Invalid media URL encoding".into()))?;

    let target_url = String::from_utf8(decoded)
        .map_err(|_| AppError::BadRequest("Invalid media URL encoding".into()))?;

    // Validate the URL is allowed (only specific domains)
    let allowed_domains = [
        "i.redd.it",
        "preview.redd.it",
        "external-preview.redd.it",
        "i.reddituploads.com",
        "v.redd.it",
        "thumbs.reddit.com",
        "emoji.reddit.com",
        "reddit.com",
        "www.reddit.com",
        "redditstatic.com",
        "styles.redditmedia.com",
        "a.thumbs.redditmedia.com",
        "b.thumbs.redditmedia.com",
    ];

    let parsed_url = match url::Url::parse(&target_url) {
        Ok(u) => u,
        Err(_) => return Err(AppError::BadRequest("Invalid target URL".into())),
    };

    let host = parsed_url.host_str().unwrap_or("");
    let is_allowed = allowed_domains.iter().any(|d| host == *d || host.ends_with(&format!(".{d}")));

    if !is_allowed {
        warn!("Blocked media proxy request to disallowed host: {host}");
        return Err(AppError::BadRequest("Media domain not allowed".into()));
    }

    // Check cache
    let cache_key = encoded_url.clone();
    if let Some(cached) = state.cache.get(&cache_key).await {
        debug!("Media cache hit: {host}");
        let mut resp = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", &cached.content_type)
            .header("content-length", cached.data.len().to_string())
            .header("cache-control", "public, max-age=86400")
            .header("x-murmur-cache", "hit");
        // Forward original cache headers
        for (key, value) in cached.cache_headers.iter() {
            if key == "etag" || key == "last-modified" {
                resp = resp.header(key, value);
            }
        }
        return Ok(resp.body(Body::from(cached.data.clone())).unwrap());
    }

    // Fetch from upstream
    let resp = state
        .client
        .get(&target_url)
        .header("User-Agent", "murmur/0.1.0 (media-proxy)")
        .send()
        .await
        .map_err(|e| AppError::MediaProxy(format!("Failed to fetch media: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::MediaProxy(format!("Upstream returned {status}")));
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let data = resp
        .bytes()
        .await
        .map_err(|e| AppError::MediaProxy(format!("Failed to read media body: {e}")))?;

    // Collect cache headers
    let mut cache_headers = HeaderMap::new();
    for key in &["etag", "last-modified", "cache-control"] {
        if let Some(value) = resp.headers().get(*key) {
            cache_headers.insert(*key, value.clone());
        }
    }

    // Cache it
    let cached = CachedMedia {
        data: data.clone(),
        content_type: content_type.clone(),
        cache_headers,
    };
    state.cache.insert(cache_key, cached).await;

    debug!("Media proxy: {host} -> {} bytes", data.len());

    Ok((
        StatusCode::OK,
        [
            ("content-type", &content_type),
            ("content-length", &data.len().to_string()),
            ("cache-control", "public, max-age=86400"),
            ("x-murmur-cache", "miss"),
        ],
        Body::from(data),
    )
        .into_response())
}

/// Encode a URL for media proxy
pub fn encode_media_url(url: &str) -> String {
    base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        url.as_bytes(),
    )
}
