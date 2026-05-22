use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::error::AppError;
use crate::models::SubredditSummary;
use crate::proxy::reddit_api::RedditClient;

#[derive(Debug, Deserialize)]
pub struct DiscoveryQuery {
    query: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/discover.html")]
pub struct DiscoverTemplate {
    pub config: Arc<Config>,
    pub results: Vec<SubredditSummary>,
    pub query: String,
    pub user: Option<crate::models::UserSession>,
}

pub async fn discover(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Query(query): Query<DiscoveryQuery>,
) -> Result<impl IntoResponse, AppError> {
    let q = query.query.unwrap_or_default();
    let results = if q.is_empty() {
        vec![]
    } else {
        reddit.search_subreddits(&q).await?
    };

    let template = DiscoverTemplate {
        config,
        results,
        query: q,
        user: None,
    };

    Html(
        template
            .render()
            .map_err(|e| AppError::Internal(format!("Template error: {e}")))?,
    )
    .into_response()
}
