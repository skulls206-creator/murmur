use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditUser {
    pub id: String,
    pub fullname: String,
    pub name: String,
    pub created_utc: f64,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub total_karma: i64,
    pub has_verified_email: bool,
    pub is_gold: bool,
    pub is_mod: bool,
    pub is_employee: bool,
    pub icon_img: Option<String>,
    pub subreddit: Option<UserSubreddit>,
    pub profile_over_18: bool,
    pub description: Option<String>,
    pub description_html: Option<String>,
    pub over_18: bool,
    pub accept_followers: bool,
    pub hide_from_robots: bool,
    pub pref_show_snoovatar: bool,
    pub has_subscribed: bool,
    pub num_friends: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubreddit {
    pub name: String,
    pub title: String,
    pub subscribers: i64,
    pub description: Option<String>,
    pub public_description: Option<String>,
    pub over_18: bool,
    pub icon_img: Option<String>,
    pub banner_img: Option<String>,
    pub primary_color: Option<String>,
    pub key_color: Option<String>,
}

/// Used for OAuth token storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<Utc>,
    pub scope: Vec<String>,
    pub mod_subreddits: Vec<String>,
}

/// Inbox / notification item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub fullname: String,
    pub author: String,
    pub subject: Option<String>,
    pub body: String,
    pub body_html: Option<String>,
    pub created_utc: f64,
    pub was_comment: bool,
    pub context: String, // permalink to the post/comment
    pub subreddit: Option<String>,
    pub new: bool,
    pub dest: String, // recipient
    pub replies: Vec<Message>,
}
