use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use cookie::{Cookie, Key, PrivateJar};
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tracing::info;

use crate::config::Config;
use crate::error::AppError;
use crate::middleware::auth::{create_session_cookie, clear_session_cookie, AuthenticatedUser};
use crate::models::UserSession;
use crate::proxy::reddit_api::RedditClient;

#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    code: String,
    state: String,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {
    pub config: Arc<Config>,
    pub redirect_to: Option<String>,
    pub user: Option<crate::models::UserSession>,
}

#[derive(Template)]
#[template(path = "pages/logout.html")]
pub struct LogoutTemplate;

/// Start Reddit OAuth2 flow
pub async fn login(
    State(config): State<Arc<Config>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let redirect_to = params.get("redirect").cloned();

    let auth_url = AuthUrl::new("https://www.reddit.com/api/v1/authorize".into())
        .map_err(|e| AppError::Internal(format!("Invalid auth URL: {e}")))?;

    let token_url = TokenUrl::new("https://www.reddit.com/api/v1/access_token".into())
        .map_err(|e| AppError::Internal(format!("Invalid token URL: {e}")))?;

    let client = oauth2::basic::BasicClient::new(
        ClientId::new(config.reddit_client_id.clone()),
        Some(ClientSecret::new(config.reddit_client_secret.clone())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!("{}auth/callback", config.base_url))
            .map_err(|e| AppError::Internal(format!("Invalid redirect URL: {e}")))?,
    );

    // Generate the authorization URL
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identity".into()))
        .add_scope(Scope::new("read".into()))
        .add_scope(Scope::new("vote".into()))
        .add_scope(Scope::new("submit".into()))
        .add_scope(Scope::new("mysubreddits".into()))
        .add_scope(Scope::new("modcontributors".into()))
        .add_scope(Scope::new("modconfig".into()))
        .add_scope(Scope::new("modposts".into()))
        .add_scope(Scope::new("modlog".into()))
        .add_scope(Scope::new("privatemessages".into()))
        .add_scope(Scope::new("report".into()))
        .add_scope(Scope::new("save".into()))
        .add_scope(Scope::new("subscribe".into()))
        .add_scope(Scope::new("flair".into()))
        .add_scope(Scope::new("structuredstyles".into()))
        .url();

    // Store CSRF token in a cookie for verification
    let csrf_cookie = Cookie::build(("murmur_csrf", csrf_token.secret().clone()))
        .http_only(true)
        .secure(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::Duration::minutes(10))
        .build();

    // Store redirect_to in a cookie if present
    let response = axum::response::Response::builder()
        .header("Location", auth_url.as_str())
        .header(
            "Set-Cookie",
            csrf_cookie.to_string(),
        );

    let response = if let Some(redirect) = redirect_to {
        let redirect_cookie = Cookie::build(("murmur_redirect", redirect))
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Lax)
            .path("/")
            .max_age(cookie::Duration::minutes(10))
            .build();
        response.header("Set-Cookie", redirect_cookie.to_string())
    } else {
        response
    };

    Ok(response.status(303).body(axum::body::Body::empty()).unwrap())
}

/// Handle OAuth2 callback from Reddit
pub async fn callback(
    State(config): State<Arc<Config>>,
    State(reddit): State<Arc<RedditClient>>,
    axum::extract::Query(query): axum::extract::Query<OAuthCallback>,
    cookie_jar: cookie::CookieJar,
) -> Result<impl IntoResponse, AppError> {
    // Verify CSRF token
    let stored_csrf = cookie_jar
        .get("murmur_csrf")
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Auth("Missing CSRF token".into()))?;

    if stored_csrf != query.state {
        return Err(AppError::Auth("CSRF token mismatch".into()));
    }

    // Exchange authorization code for tokens
    let auth_url = AuthUrl::new("https://www.reddit.com/api/v1/authorize".into())
        .map_err(|e| AppError::Internal(format!("Invalid auth URL: {e}")))?;
    let token_url = TokenUrl::new("https://www.reddit.com/api/v1/access_token".into())
        .map_err(|e| AppError::Internal(format!("Invalid token URL: {e}")))?;

    let client = oauth2::basic::BasicClient::new(
        ClientId::new(config.reddit_client_id.clone()),
        Some(ClientSecret::new(config.reddit_client_secret.clone())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!("{}auth/callback", config.base_url))
            .map_err(|e| AppError::Internal(format!("Invalid redirect URL: {e}")))?,
    );

    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(|request| async {
            let client = reqwest::Client::new();
            let resp = client
                .request(request.method().clone(), request.url().clone())
                .headers(request.headers().clone())
                .body(request.body().clone())
                .send()
                .await
                .map_err(|e| oauth2::RequestTokenError::Other(e.into()))?;
            Ok(resp)
        })
        .await
        .map_err(|e| AppError::Auth(format!("Token exchange failed: {e}")))?;

    let access_token = token_result.access_token().secret().clone();
    let refresh_token = token_result.refresh_token().map(|t| t.secret().clone());

    // Get user identity
    let user = reddit.get_user_identity(&access_token).await?;

    let session = UserSession {
        user_id: user.id.clone(),
        username: user.name.clone(),
        access_token,
        refresh_token,
        expires_at: chrono::Utc::now()
            + chrono::Duration::seconds(token_result.expires_in().unwrap_or(3600) as i64),
        scope: vec![], // TODO: parse from token response
        mod_subreddits: vec![],
    };

    // Create session cookie
    let key = Key::from(config.cookie_secret.as_bytes());
    let session_cookie = create_session_cookie(&session, &key);

    // Determine redirect destination
    let redirect_to = cookie_jar
        .get("murmur_redirect")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "/".to_string());

    info!("User logged in: {} ({})", user.name, user.id);

    let response = axum::response::Response::builder()
        .header("Location", &redirect_to)
        .header("Set-Cookie", session_cookie.to_string())
        .header(
            "Set-Cookie",
            Cookie::build(("murmur_csrf", ""))
                .path("/")
                .max_age(cookie::Duration::seconds(0))
                .build()
                .to_string(),
        )
        .header(
            "Set-Cookie",
            Cookie::build(("murmur_redirect", ""))
                .path("/")
                .max_age(cookie::Duration::seconds(0))
                .build()
                .to_string(),
        )
        .status(303)
        .body(axum::body::Body::empty())
        .unwrap();

    Ok(response)
}

/// Logout — clear session
pub async fn logout() -> impl IntoResponse {
    let clear_cookie = clear_session_cookie();
    axum::response::Response::builder()
        .header("Location", "/")
        .header("Set-Cookie", clear_cookie.to_string())
        .status(303)
        .body(axum::body::Body::empty())
        .unwrap()
}

pub async fn login_page(
    State(config): State<Arc<Config>>,
) -> impl IntoResponse {
    let template = LoginTemplate {
        config,
        redirect_to: None,
        user: None,
    };
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template error".into()),
    )
}
