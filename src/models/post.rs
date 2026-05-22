use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub fullname: String,            // "t3_xxxxx"
    pub title: String,
    pub author: String,
    pub author_fullname: Option<String>,
    pub subreddit: String,
    pub subreddit_id: Option<String>,
    pub subreddit_subscribers: Option<i64>,

    pub score: i64,
    pub upvote_ratio: f64,
    pub likes: Option<bool>,         // true=upvoted, false=downvoted, null=no vote
    pub saved: bool,
    pub stickied: bool,
    pub locked: bool,
    pub archived: bool,
    pub nsfw: bool,
    pub spoiler: bool,

    pub domain: Option<String>,
    pub url: Option<String>,
    pub permalink: String,
    pub selftext: Option<String>,
    pub selftext_html: Option<String>,

    pub thumbnail: Option<String>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,

    pub media: Option<PostMedia>,
    pub gallery: Option<Vec<GalleryItem>>,
    pub crosspost_parent: Option<String>,

    pub num_comments: i64,
    pub num_crossposts: Option<i64>,
    pub created_utc: f64,
    pub edited: bool,

    pub link_flair_text: Option<String>,
    pub link_flair_background_color: Option<String>,
    pub link_flair_richtext: Option<Vec<FlairRichtext>>,

    pub author_flair_text: Option<String>,
    pub author_flair_background_color: Option<String>,

    pub distinguished: Option<String>, // "moderator", "admin", "special"
    pub over_18: bool,
    pub is_video: bool,
    pub is_gallery: bool,
    pub is_self: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMedia {
    pub kind: MediaKind,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fallback_url: Option<String>,
    pub scrubber_media_url: Option<String>,
    pub hls_url: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaKind {
    Image,
    Video,
    Gif,
    External(String), // link to external site
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryItem {
    pub media_id: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
    pub caption: Option<String>,
    pub outbound_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlairRichtext {
    pub e: String, // "text" or "emoji"
    pub t: Option<String>,
    pub a: Option<String>, // emoji URL for "emoji" type
    pub u: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostList {
    pub posts: Vec<Post>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub dist: Option<i32>,
}
