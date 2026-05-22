use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::error::AppError;
use crate::models::{FeedView, SortMode, PostList};
use crate::proxy::reddit_api::RedditClient;

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    sort: Option<String>,
    after: Option<String>,
    before: Option<String>,
    count: Option<usize>,
    view: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/frontpage.html")]
pub struct FrontpageTemplate {
    pub config: Arc<Config>,
    pub posts: PostList,
    pub sort: SortMode,
    pub view: FeedView,
    pub user: Option<crate::models::UserSession>,
}

#[derive(Template)]
#[template(path = "partials/feed_items.html")]
pub struct FeedItemsTemplate {
    pub posts: PostList,
    pub sort: SortMode,
    pub view: FeedView,
}

pub async fn get_frontpage(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let sort = SortMode::from_str(query.sort.as_deref().unwrap_or("hot"));
    let view = FeedView::from_str(query.view.as_deref().unwrap_or("card"));
    let after = query.after.as_deref();
    let before = query.before.as_deref();
    let count = query.count.unwrap_or(0);

    let posts = reddit
        .get_frontpage(sort, after, before, count)
        .await?;

    // If it's an HTMX request, return just the feed items
    // For now, return the full page
    let template = FrontpageTemplate {
        config,
        posts,
        sort,
        view,
        user: None, // will be hydrated from auth middleware
    };

    Html(
        template
            .render()
            .map_err(|e| AppError::Internal(format!("Template error: {e}")))?,
    )
    .into_response()
}

pub async fn get_frontpage_feed_items(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let sort = SortMode::from_str(query.sort.as_deref().unwrap_or("hot"));
    let view = FeedView::from_str(query.view.as_deref().unwrap_or("card"));
    let after = query.after.as_deref();
    let before = query.before.as_deref();
    let count = query.count.unwrap_or(0);

    let posts = reddit
        .get_frontpage(sort, after, before, count)
        .await?;

    let template = FeedItemsTemplate {
        posts,
        sort,
        view,
    };

    Html(
        template
            .render()
            .map_err(|e| AppError::Internal(format!("Template error: {e}")))?,
    )
    .into_response()
}
