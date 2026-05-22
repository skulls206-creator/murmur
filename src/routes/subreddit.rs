use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::config::Config;
use crate::error::AppError;
use crate::models::{FeedView, SortMode, TimeFilter, PostList, Subreddit};
use crate::proxy::reddit_api::RedditClient;

#[derive(Debug, Deserialize)]
pub struct SubredditQuery {
    sort: Option<String>,
    t: Option<String>,
    after: Option<String>,
    before: Option<String>,
    count: Option<usize>,
    view: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/subreddit.html")]
pub struct SubredditTemplate {
    pub config: Arc<Config>,
    pub subreddit: Subreddit,
    pub posts: PostList,
    pub sort: SortMode,
    pub time: TimeFilter,
    pub view: FeedView,
    pub user: Option<crate::models::UserSession>,
}

#[derive(Template)]
#[template(path = "partials/subreddit_sidebar.html")]
pub struct SubredditSidebarTemplate {
    pub subreddit: Subreddit,
}

pub async fn get_subreddit(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    Path(subreddit_name): Path<String>,
    Query(query): Query<SubredditQuery>,
) -> Result<impl IntoResponse, AppError> {
    let sort = SortMode::from_str(query.sort.as_deref().unwrap_or("hot"));
    let time = match sort {
        SortMode::Top | SortMode::Controversial => {
            TimeFilter::from_str(query.t.as_deref().unwrap_or("day"))
        }
        _ => TimeFilter::Day, // unused for non-time-based sorts
    };
    let view = FeedView::from_str(query.view.as_deref().unwrap_or("card"));
    let after = query.after.as_deref();
    let before = query.before.as_deref();
    let count = query.count.unwrap_or(0);

    // Fetch subreddit info + posts in parallel
    let (subreddit, posts) = tokio::join!(
        reddit.get_subreddit_info(&subreddit_name),
        reddit.get_subreddit(
            &subreddit_name,
            sort,
            if matches!(sort, SortMode::Top | SortMode::Controversial) { Some(time) } else { None },
            after,
            before,
            count,
        ),
    );

    let subreddit = subreddit?;
    let posts = posts?;

    let template = SubredditTemplate {
        config,
        subreddit,
        posts,
        sort,
        time,
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

impl TimeFilter {
    fn from_str(s: &str) -> Self {
        match s {
            "hour" => TimeFilter::Hour,
            "day" => TimeFilter::Day,
            "week" => TimeFilter::Week,
            "month" => TimeFilter::Month,
            "year" => TimeFilter::Year,
            "all" => TimeFilter::All,
            _ => TimeFilter::Day,
        }
    }
}
