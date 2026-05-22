use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use cookie::{Cookie, CookieJar, Key, PrivateJar};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::Config;
use crate::error::AppError;
use crate::models::UserSession;

/// Auth middleware — extracts and validates user session from cookie
///
/// If the user has a valid session cookie, we hydrate the request
/// extensions with their user info. Otherwise, they're anonymous.
///
/// This middleware does NOT block unauthenticated users unless
/// require_login is set in the instance config.

/// Session data stored in the encrypted cookie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session: UserSession,
}

/// Extension carrying the authenticated user (None if anonymous)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub Option<UserSession>);

/// Extract session from cookie jar
pub fn get_session_from_cookies(cookie_jar: &CookieJar, key: &Key) -> Option<UserSession> {
    let private = cookie_jar.private(key);
    if let Some(cookie) = private.get("murmur_session") {
        let value = cookie.value();
        // Decode base64+JSON
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, value)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SessionData>(&bytes).ok())
        {
            Some(data) => Some(data.session),
            None => {
                warn!("Failed to decode session cookie");
                None
            }
        }
    } else {
        None
    }
}

/// Create a session cookie
pub fn create_session_cookie(session: &UserSession, key: &Key) -> Cookie<'static> {
    let data = SessionData {
        session: session.clone(),
    };
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &serde_json::to_vec(&data).expect("Failed to serialize session"),
    );

    let mut cookie = Cookie::new("murmur_session", encoded);
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(cookie::SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(cookie::Duration::days(30));

    let mut jar = CookieJar::new();
    jar.private(key).add(cookie);

    // Return the cookie from the private jar
    jar.get("murmur_session").unwrap().clone()
}

/// Remove the session cookie
pub fn clear_session_cookie<'a>() -> Cookie<'a> {
    let mut cookie = Cookie::new("murmur_session", "");
    cookie.set_path("/");
    cookie.set_max_age(cookie::Duration::seconds(0));
    cookie
}

/// Auth middleware layer
pub async fn auth_middleware(
    Extension(config): Extension<Arc<Config>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = Key::from(config.cookie_secret.as_bytes());
    let cookie_header = request
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let jar = CookieJar::new();
    let jar = if let Some(cookie) = parse_cookie_header(cookie_header) {
        jar.add(cookie)
    } else {
        jar
    };

    let user = get_session_from_cookies(&jar, &key);

    // If require_login is set and user is anonymous, block them
    if config.require_login && user.is_none() {
        return Err(AppError::Unauthorized);
    }

    request.extensions_mut().insert(AuthenticatedUser(user));
    request.extensions_mut().insert(jar);

    Ok(next.run(request).await)
}

fn parse_cookie_header(header: &str) -> Option<Cookie<'static>> {
    // Simple cookie parser — just extract murmur_session
    for pair in header.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix("murmur_session=") {
            return Some(Cookie::new("murmur_session", value.to_string()));
        }
    }
    None
}
