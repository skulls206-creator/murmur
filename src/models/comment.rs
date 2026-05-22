use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub fullname: String,
    pub author: String,
    pub author_fullname: Option<String>,
    pub subreddit: String,
    pub parent_id: Option<String>,  // fullname of parent
    pub link_id: String,           // fullname of parent post
    pub body: String,
    pub body_html: Option<String>,
    pub score: i64,
    pub likes: Option<bool>,
    pub saved: bool,
    pub stickied: bool,
    pub locked: bool,
    pub archived: bool,
    pub nsfw: bool,
    pub edited: bool,
    pub created_utc: f64,
    pub depth: i32,
    pub collapsed: bool,
    pub is_submitter: bool,
    pub distinguished: Option<String>,
    pub author_flair_text: Option<String>,
    pub author_flair_background_color: Option<String>,
    pub replies: Vec<Comment>,
    pub controversiality: Option<i64>,
    pub gilded: i32,
    pub score_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentList {
    pub comments: Vec<Comment>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub dist: Option<i32>,
}

/// A single comment tree (post + its comments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentTree {
    pub post_id: String,
    pub post_fullname: String,
    pub comments: Vec<Comment>,
    pub more: Option<MoreComments>,
}

/// Reddit "more" comments placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoreComments {
    pub count: i32,
    pub children: Vec<String>,
    pub parent_id: String,
    pub depth: i32,
}
