use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subreddit {
    pub id: String,
    pub fullname: String,
    pub name: String,              // "programming" without r/
    pub title: String,
    pub description: Option<String>,
    pub description_html: Option<String>,
    pub public_description: Option<String>,
    pub sidebar: Option<String>,
    pub sidebar_html: Option<String>,
    pub icon_img: Option<String>,
    pub banner_img: Option<String>,
    pub community_icon: Option<String>,
    pub primary_color: Option<String>,
    pub key_color: Option<String>,
    pub subscribers: i64,
    pub active_user_count: Option<i64>,
    pub created_utc: f64,
    pub over_18: bool,
    pub user_is_subscriber: Option<bool>,
    pub user_is_moderator: Option<bool>,
    pub user_flair_text: Option<String>,
    pub user_flair_background_color: Option<String>,
    pub submit_text: Option<String>,
    pub submit_text_html: Option<String>,
    pub submit_text_label: Option<String>,
    pub link_flair_enabled: bool,
    pub link_flair_position: Option<String>, // "left", "right", or ""
    pub user_flair_enabled_in_sidebar: bool,
    pub lang: String,
    pub spoilers_enabled: bool,
    pub allow_videos: bool,
    pub allow_images: bool,
    pub allow_galleries: bool,
    pub wiki_enabled: bool,
    pub hide_ads: bool,
    pub comment_score_hide_mins: i32,
    pub suggested_comment_sort: Option<String>,
    pub rules: Vec<SubredditRule>,
    pub moderators: Option<Vec<Moderator>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditRule {
    pub kind: String, // "link" or "comment"
    pub short_name: String,
    pub description: String,
    pub description_html: Option<String>,
    pub created_utc: f64,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moderator {
    pub name: String,
    pub mod_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditList {
    pub subreddits: Vec<SubredditSummary>,
    pub after: Option<String>,
    pub before: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditSummary {
    pub name: String,
    pub title: String,
    pub subscribers: i64,
    pub active_user_count: Option<i64>,
    pub description: Option<String>,
    pub icon_img: Option<String>,
    pub community_icon: Option<String>,
    pub primary_color: Option<String>,
    pub over_18: bool,
    pub url: String, // "/r/programming"
}
