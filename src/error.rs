use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Reddit API error: {0}")]
    RedditApi(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited. Try again in {0} seconds.")]
    RateLimited(u64),

    #[error("NSFW content is disabled on this instance")]
    NsfwBlocked,

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Upstream rate limited — Reddit API is throttling us")]
    UpstreamRateLimited,

    #[error("Media proxy failed: {0}")]
    MediaProxy(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::RateLimited(retry_after) => {
                let resp = (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("Retry-After", &retry_after.to_string())],
                    Json(json!({
                        "error": "rate_limited",
                        "message": self.to_string(),
                        "retry_after": retry_after,
                    })),
                );
                return resp.into_response();
            }
            AppError::NsfwBlocked => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::UpstreamRateLimited => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server configuration error".into()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::MediaProxy(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".into()),
        };

        (
            status,
            Json(json!({
                "error": status.canonical_reason().unwrap_or("unknown"),
                "message": message,
            })),
        ).into_response()
    }
}
