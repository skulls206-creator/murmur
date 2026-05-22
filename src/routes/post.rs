use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::error::AppError;
use crate::models::{Comment, Post};
use crate::proxy::reddit_api::RedditClient;

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    sort: Option<String>,
    limit: Option<usize>,
}

#[derive(Template)]
#[template(path = "pages/post.html")]
pub struct PostTemplate {
    pub config: Arc<Config>,
    pub post: Post,
    pub comments: Vec<Comment>,
    pub comment_sort: String,
    pub user: Option<crate::models::UserSession>,
}

#[derive(Template)]
#[template(path = "partials/comment_thread.html")]
pub struct CommentThreadTemplate {
    pub comments: Vec<Comment>,
    pub depth: i32,
}

pub async fn get_post(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Path((subreddit, post_id)): Path<(String, String)>,
    Query(query): Query<PostQuery>,
) -> Result<impl IntoResponse, AppError> {
    let comment_sort = query.sort.as_deref().unwrap_or("top").to_string();
    let limit = query.limit.or(Some(200));

    let (post, comments) = reddit
        .get_post_and_comments(&subreddit, &post_id, Some(&comment_sort), limit)
        .await?;

    let template = PostTemplate {
        config,
        post,
        comments,
        comment_sort,
        user: None,
    };

    Html(
        template
            .render()
            .map_err(|e| AppError::Internal(format!("Template error: {e}")))?,
    )
    .into_response()
}
