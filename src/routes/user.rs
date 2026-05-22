use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::error::AppError;
use crate::models::{FeedView, PostList, RedditUser, SortMode};
use crate::proxy::reddit_api::RedditClient;

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    sort: Option<String>,
    after: Option<String>,
    before: Option<String>,
    count: Option<usize>,
    view: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/user.html")]
pub struct UserTemplate {
    pub config: Arc<Config>,
    pub profile: RedditUser,
    pub posts: PostList,
    pub sort: SortMode,
    pub view: FeedView,
    pub user: Option<crate::models::UserSession>,
}

pub async fn get_user(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Path(username): Path<String>,
    Query(query): Query<UserQuery>,
) -> Result<impl IntoResponse, AppError> {
    let sort = SortMode::from_str(query.sort.as_deref().unwrap_or("hot"));
    let view = FeedView::from_str(query.view.as_deref().unwrap_or("card"));
    let after = query.after.as_deref();
    let before = query.before.as_deref();
    let count = query.count.unwrap_or(0);

    let (profile, posts) = tokio::join!(
        reddit.get_user(&username),
        reddit.get_user_posts(&username, sort, after, before, count),
    );

    let profile = profile?;
    let posts = posts?;

    let template = UserTemplate {
        config,
        profile,
        posts,
        sort,
        view,
        user: None,
    };

    Html(
        template
            .render()
            .map_err(|e| AppError::Internal(format!("Template error: {e}")))?,
    )
    .into_response()
}
