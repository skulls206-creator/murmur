use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::error::AppError;
use crate::models::PostList;
use crate::proxy::reddit_api::{RedditClient, SearchType};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    r#type: Option<String>, // "posts" or "subreddits"
    after: Option<String>,
    before: Option<String>,
    count: Option<usize>,
}

#[derive(Template)]
#[template(path = "pages/search.html")]
pub struct SearchTemplate {
    pub config: Arc<Config>,
    pub query: String,
    pub search_type: String,
    pub results: Option<PostList>,
    pub user: Option<crate::models::UserSession>,
}

pub async fn search(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let search_type = match query.r#type.as_deref() {
        Some("subreddits") => SearchType::Subreddits,
        _ => SearchType::Posts,
    };

    let after = query.after.as_deref();
    let before = query.before.as_deref();
    let count = query.count.unwrap_or(0);

    let results = if query.q.is_empty() {
        None
    } else {
        let results = reddit
            .search(&query.q, search_type, after, before, count)
            .await?;
        Some(results)
    };

    let template = SearchTemplate {
        config,
        query: query.q,
        search_type: match search_type {
            SearchType::Posts => "posts".into(),
            SearchType::Subreddits => "subreddits".into(),
        },
        results,
        user: None,
    };

    Html(
        template
            .render()
            .map_err(|e| AppError::Internal(format!("Template error: {e}")))?,
    )
    .into_response()
}
