use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::config::Config;
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::proxy::reddit_api::RedditClient;

/// JSON API endpoints for authenticated actions
/// These are consumed by the progressive-enhancement JavaScript

#[derive(Debug, Deserialize)]
pub struct VoteBody {
    id: String,     // fullname like "t3_abc123" or "t1_def456"
    dir: i8,        // 1 = upvote, -1 = downvote, 0 = unvote
}

#[derive(Debug, Deserialize)]
pub struct CommentBody {
    parent: String, // fullname of post or comment
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitBody {
    subreddit: String,
    title: String,
    kind: String,       // "self", "link", "image", "video"
    text: Option<String>,
    url: Option<String>,
}

/// Vote on a post or comment
pub async fn api_vote(
    State(reddit): State<Arc<RedditClient>>,
    auth: AuthenticatedUser,
    Json(body): Json<VoteBody>,
) -> Result<impl IntoResponse, AppError> {
    let session = auth.0.ok_or(AppError::Unauthorized)?;

    if body.dir < -1 || body.dir > 1 {
        return Err(AppError::BadRequest("Direction must be -1, 0, or 1".into()));
    }

    reddit.vote(&body.id, body.dir, &session.access_token).await?;

    Ok(Json(json!({
        "success": true,
        "id": body.id,
        "dir": body.dir,
    })))
}

/// Submit a comment
pub async fn api_comment(
    State(reddit): State<Arc<RedditClient>>,
    auth: AuthenticatedUser,
    Json(body): Json<CommentBody>,
) -> Result<impl IntoResponse, AppError> {
    let session = auth.0.ok_or(AppError::Unauthorized)?;

    if body.text.trim().is_empty() {
        return Err(AppError::BadRequest("Comment text is required".into()));
    }

    let comment = reddit
        .submit_comment(&body.parent, &body.text, &session.access_token)
        .await?;

    Ok(Json(json!({
        "success": true,
        "comment": comment,
    })))
}

/// Health check
pub async fn api_health(
    State(reddit): State<Arc<RedditClient>>,
) -> Result<impl IntoResponse, AppError> {
    // Try to reach Reddit API
    reddit.ensure_token().await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    ))
}
