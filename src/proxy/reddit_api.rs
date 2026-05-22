use std::sync::Arc;
use std::time::Duration;

use axum::http::HeaderMap;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING, USER_AGENT};
use reqwest::Client;
use serde_json::Value;

use crate::config::Config;
use crate::error::AppError;
use crate::models::*;

/// Reddit API client — all requests go through here
///
/// Architecture:
///   User -> Murmur -> Reddit API (as Reddit OAuth app) -> Response -> Murmur -> User
///
/// All requests are made with a fixed User-Agent, no cookies forwarded from
/// the user's browser, and no tracking headers preserved.
pub struct RedditClient {
    client: Client,
    base_url: String,
    access_token: tokio::sync::RwLock<Option<String>>,
    config: Arc<Config>,
}

impl RedditClient {
    pub fn new(config: Arc<Config>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "web:murmur:v0.1.0 (by /u/murmur-app)"
            ),
        );

        let client_builder = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.upstream_timeout_seconds))
            .gzip(true)
            .brotli(true)
            .cookie_store(true); // for Reddit's cookie-based auth fallback

        // Optional SOCKS5 proxy for Tor/I2P
        let client_builder = if let Some(socks5) = &config.socks5_proxy {
            let proxy = reqwest::Proxy::all(socks5)
                .expect("Invalid SOCKS5 proxy URL");
            client_builder.proxy(proxy)
        } else {
            client_builder
        };

        Self {
            client: client_builder.build().expect("Failed to build reqwest Client"),
            base_url: "https://oauth.reddit.com".into(),
            access_token: tokio::sync::RwLock::new(None),
            config,
        }
    }

    /// Ensure we have a valid access token (app-only OAuth for logged-out use)
    pub async fn ensure_token(&self) -> Result<String, AppError> {
        let token = self.access_token.read().await;
        if let Some(token) = token.as_ref() {
            return Ok(token.clone());
        }
        drop(token);

        self.refresh_app_token().await
    }

    /// Fetch an application-only OAuth token (no user context)
    async fn refresh_app_token(&self) -> Result<String, AppError> {
        let params = [
            ("grant_type", "client_credentials"),
            ("duration", "permanent"),
        ];

        let resp = self
            .client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(&self.config.reddit_client_id, Some(&self.config.reddit_client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to get app token: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse token response: {e}")))?;

        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| AppError::RedditApi("No access_token in response".into()))?
            .to_string();

        let mut stored = self.access_token.write().await;
        *stored = Some(token.clone());
        Ok(token)
    }

    // ──── Feed / Frontpage ────

    pub async fn get_frontpage(
        &self,
        sort: SortMode,
        after: Option<&str>,
        before: Option<&str>,
        count: usize,
    ) -> Result<PostList, AppError> {
        let token = self.ensure_token().await?;
        let mut url = format!("{}/{}", self.base_url, sort.as_str());
        let mut params = vec![("limit", "25"), ("raw_json", "1")];
        if let Some(a) = after { params.push(("after", a)); }
        if let Some(b) = before { params.push(("before", b)); }
        params.push(("count", &count.to_string()));

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Frontpage fetch failed: {e}")))?;

        Self::parse_listing(resp).await
    }

    pub async fn get_subreddit(
        &self,
        subreddit: &str,
        sort: SortMode,
        time: Option<TimeFilter>,
        after: Option<&str>,
        before: Option<&str>,
        count: usize,
    ) -> Result<PostList, AppError> {
        let token = self.ensure_token().await?;
        let mut url = format!("{}/r/{}/{}", self.base_url, subreddit, sort.as_str());
        let mut params = vec![("limit", "25"), ("raw_json", "1")];

        if let Some(t) = time {
            params.push(("t", t.as_str()));
        }
        if let Some(a) = after { params.push(("after", a)); }
        if let Some(b) = before { params.push(("before", b)); }
        params.push(("count", &count.to_string()));

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Subreddit fetch failed: {e}")))?;

        Self::parse_listing(resp).await
    }

    pub async fn get_post_and_comments(
        &self,
        subreddit: &str,
        post_id: &str,
        sort: Option<&str>,
        limit: Option<usize>,
    ) -> Result<(Post, Vec<Comment>), AppError> {
        let token = self.ensure_token().await?;
        let sort = sort.unwrap_or("top");
        let limit = limit.unwrap_or(200);

        let url = format!("{}/r/{}/comments/{}/", self.base_url, subreddit, post_id);
        let params = [
            ("limit", &limit.to_string()),
            ("sort", sort),
            ("raw_json", "1"),
            ("depth", "8"),
            ("threaded", "1"),
        ];

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Post fetch failed: {e}")))?;

        let body: Vec<Value> = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse post response: {e}")))?;

        if body.len() < 2 {
            return Err(AppError::NotFound("Post not found".into()));
        }

        let post_data = &body[0]["data"]["children"][0]["data"];
        let post: Post = serde_json::from_value(post_data.clone())
            .map_err(|e| AppError::Internal(format!("Failed to deserialize post: {e}")))?;

        let comments_data = &body[1]["data"]["children"];
        let comments: Vec<Comment> = Self::parse_comments_recursive(comments_data);

        Ok((post, comments))
    }

    // ──── User ────

    pub async fn get_user(
        &self,
        username: &str,
    ) -> Result<RedditUser, AppError> {
        let token = self.ensure_token().await?;
        let url = format!("{}/user/{}/about", self.base_url, username);

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("User fetch failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse user response: {e}")))?;

        serde_json::from_value(body["data"].clone())
            .map_err(|e| AppError::Internal(format!("Failed to deserialize user: {e}")))
    }

    pub async fn get_user_posts(
        &self,
        username: &str,
        sort: SortMode,
        after: Option<&str>,
        before: Option<&str>,
        count: usize,
    ) -> Result<PostList, AppError> {
        let token = self.ensure_token().await?;
        let mut url = format!("{}/user/{}/submitted", self.base_url, username);
        let mut params = vec![("limit", "25"), ("raw_json", "1")];
        params.push(("sort", sort.as_str()));
        if let Some(a) = after { params.push(("after", a)); }
        if let Some(b) = before { params.push(("before", b)); }
        params.push(("count", &count.to_string()));

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("User posts fetch failed: {e}")))?;

        Self::parse_listing(resp).await
    }

    // ──── Search ────

    pub async fn search(
        &self,
        query: &str,
        kind: SearchType,
        after: Option<&str>,
        before: Option<&str>,
        count: usize,
    ) -> Result<PostList, AppError> {
        let token = self.ensure_token().await?;
        let endpoint = match kind {
            SearchType::Posts => "search",
            SearchType::Subreddits => "subreddits/search",
        };
        let url = format!("{}/{}", self.base_url, endpoint);
        let mut params = vec![("q", query), ("limit", "25"), ("raw_json", "1")];
        if let Some(a) = after { params.push(("after", a)); }
        if let Some(b) = before { params.push(("before", b)); }
        params.push(("count", &count.to_string()));

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Search failed: {e}")))?;

        Self::parse_listing(resp).await
    }

    pub async fn search_subreddits(
        &self,
        query: &str,
    ) -> Result<Vec<SubredditSummary>, AppError> {
        let token = self.ensure_token().await?;
        let url = format!("{}/api/subreddit_autocomplete_v2", self.base_url);
        let params = [
            ("q", query),
            ("limit", "10"),
            ("include_profiles", "on"),
        ];

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Subreddit search failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse subreddit search: {e}")))?;

        let subreddits = body["data"]["children"]
            .as_array()
            .map(|children| {
                children
                    .iter()
                    .filter_map(|c| {
                        let data = &c["data"];
                        serde_json::from_value(data.clone()).ok()
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(subreddits)
    }

    // ──── Subreddit Info ────

    pub async fn get_subreddit_info(&self, subreddit: &str) -> Result<Subreddit, AppError> {
        let token = self.ensure_token().await?;
        let url = format!("{}/r/{}/about", self.base_url, subreddit);

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Subreddit info failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse subreddit info: {e}")))?;

        serde_json::from_value(body["data"].clone())
            .map_err(|e| AppError::Internal(format!("Failed to deserialize subreddit: {e}")))
    }

    // ──── Vote ────

    pub async fn vote(&self, fullname: &str, direction: i8, access_token: &str) -> Result<(), AppError> {
        let url = format!("{}/api/vote", self.base_url);
        let params = [
            ("id", fullname),
            ("dir", &direction.to_string()),
        ];

        let resp = self
            .client
            .post(&url)
            .form(&params)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Vote failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::RedditApi(format!("Vote returned {status}: {text}")));
        }

        Ok(())
    }

    // ──── Comment ────

    pub async fn submit_comment(
        &self,
        parent: &str,
        text: &str,
        access_token: &str,
    ) -> Result<Comment, AppError> {
        let url = format!("{}/api/comment", self.base_url);
        let params = [
            ("thing_id", parent),
            ("text", text),
            ("api_type", "json"),
        ];

        let resp = self
            .client
            .post(&url)
            .form(&params)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Comment failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse comment response: {e}")))?;

        // Reddit returns errors in json.errors[]
        if let Some(errors) = body["json"]["errors"].as_array() {
            if !errors.is_empty() {
                let msg = errors
                    .iter()
                    .filter_map(|e| e.as_array().and_then(|a| a.get(1).and_then(|v| v.as_str())))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(AppError::RedditApi(format!("Comment rejected: {msg}")));
            }
        }

        let comment_data = &body["json"]["data"]["things"][0]["data"];
        serde_json::from_value(comment_data.clone())
            .map_err(|e| AppError::Internal(format!("Failed to deserialize comment: {e}")))
    }

    // ──── Submit Post ────

    pub async fn submit_post(
        &self,
        subreddit: &str,
        title: &str,
        kind: SubmissionKind,
        text: Option<&str>,
        url: Option<&str>,
        access_token: &str,
    ) -> Result<String, AppError> {
        let endpoint = format!("{}/api/submit", self.base_url);
        let kind_str = match kind {
            SubmissionKind::Text => "self",
            SubmissionKind::Link => "link",
            SubmissionKind::Image => "image",
            SubmissionKind::Video => "video",
        };

        let mut params = vec![
            ("sr", subreddit),
            ("title", title),
            ("kind", kind_str),
            ("api_type", "json"),
        ];
        if let Some(t) = text { params.push(("text", t)); }
        if let Some(u) = url { params.push(("url", u)); }

        let resp = self
            .client
            .post(&endpoint)
            .form(&params)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Submit failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse submit response: {e}")))?;

        if let Some(errors) = body["json"]["errors"].as_array() {
            if !errors.is_empty() {
                let msg = errors
                    .iter()
                    .filter_map(|e| e.as_array().and_then(|a| a.get(1).and_then(|v| v.as_str())))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(AppError::RedditApi(format!("Submit rejected: {msg}")));
            }
        }

        body["json"]["data"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::RedditApi("No post id returned".into()))
    }

    // ──── Auth ────

    pub async fn refresh_user_token(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String, usize), AppError> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let resp = self
            .client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(&self.config.reddit_client_id, Some(&self.config.reddit_client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Auth(format!("Token refresh failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Auth(format!("Failed to parse refresh response: {e}")))?;

        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| AppError::Auth("No access_token in refresh".into()))?
            .to_string();

        let refresh = body["refresh_token"]
            .as_str()
            .unwrap_or(refresh_token)
            .to_string();

        let expires_in = body["expires_in"].as_u64().unwrap_or(3600) as usize;

        Ok((token, refresh, expires_in))
    }

    pub async fn get_user_identity(&self, access_token: &str) -> Result<RedditUser, AppError> {
        let url = format!("{}/api/v1/me", self.base_url);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::RedditApi(format!("Identity fetch failed: {e}")))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse identity: {e}")))?;

        serde_json::from_value(body)
            .map_err(|e| AppError::Internal(format!("Failed to deserialize identity: {e}")))
    }

    // ──── Helpers ────

    async fn parse_listing(resp: reqwest::Response) -> Result<PostList, AppError> {
        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::RedditApi(format!("Failed to parse listing: {e}")))?;

        let data = &body["data"];
        let after = data["after"].as_str().map(String::from);
        let before = data["before"].as_str().map(String::from);
        let dist = data["dist"].as_i64().map(|d| d as i32);

        let posts: Vec<Post> = data["children"]
            .as_array()
            .map(|children| {
                children
                    .iter()
                    .filter_map(|c| {
                        let kind = c["kind"].as_str().unwrap_or("");
                        if kind != "t3" {
                            return None; // skip comments and other types
                        }
                        serde_json::from_value(c["data"].clone()).ok()
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(PostList {
            posts,
            after,
            before,
            dist,
        })
    }

    fn parse_comments_recursive(children: &[Value]) -> Vec<Comment> {
        let mut comments = Vec::new();

        for child in children {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "t1" => {
                    let data = &child["data"];
                    if let Ok(mut comment) = serde_json::from_value::<Comment>(data.clone()) {
                        if let Some(replies_data) = data["replies"].as_object() {
                            if let Some(replies_children) = replies_data["data"]["children"].as_array() {
                                comment.replies = Self::parse_comments_recursive(replies_children);
                            }
                        }
                        comments.push(comment);
                    }
                }
                "more" => {
                    // "more" comments — we store them inline in the parent
                    // or return separately — for now, skip to keep things simple
                    continue;
                }
                _ => {}
            }
        }

        comments
    }
}

#[derive(Debug, Clone)]
pub enum SearchType {
    Posts,
    Subreddits,
}

#[derive(Debug, Clone)]
pub enum SubmissionKind {
    Text,
    Link,
    Image,
    Video,
}
