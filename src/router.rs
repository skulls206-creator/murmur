use std::sync::Arc;

use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::middleware::cache::CacheLayer;
use crate::middleware::csp::{csp_header, security_headers};
use crate::proxy::media_proxy::{self, MediaProxyState};
use crate::proxy::reddit_api::RedditClient;
use crate::routes;

/// Build the application router with all middleware and routes
pub fn build_router(config: Arc<Config>) -> Router {
    // Initialize shared state
    let reddit: Arc<RedditClient> = Arc::new(RedditClient::new(config.clone()));
    let media_proxy: Arc<MediaProxyState> = Arc::new(MediaProxyState::new(config.clone()));
    let cache_layer: Arc<CacheLayer> = Arc::new(CacheLayer::new(config.cache_ttl_seconds));

    // Static file serving (CSS, JS, fonts)
    let static_router = Router::new()
        .nest_service(
            "/static",
            tower_http::services::ServeDir::new("static")
                .precompressed_gzip()
                .cache_control_headers(|path| {
                    if path.ends_with(".css") || path.ends_with(".js") {
                        tower_http::services::fs::DefaultCacheControl::new()
                    } else {
                        tower_http::services::fs::DefaultCacheControl::new()
                    }
                }),
        )
        // Fallback: the compiled index.html / robots.txt etc
        .nest_service("/favicon.ico", tower_http::services::ServeFile::new("static/favicon.ico"))
        .nest_service("/robots.txt", tower_http::services::ServeFile::new("static/robots.txt"))
        .nest_service("/manifest.json", tower_http::services::ServeFile::new("static/manifest.json"));

    // Configure media proxy routes
    let media_routes = Router::new()
        .route("/media/{*encoded_url}", get(media_proxy::proxy_media));

    // Front page routes
    let frontpage_routes = Router::new()
        .route("/", get(routes::frontpage::get_frontpage))
        .route("/feed", get(routes::frontpage::get_frontpage_feed_items));

    // Subreddit routes
    let subreddit_routes = Router::new()
        .route("/r/{subreddit}", get(routes::subreddit::get_subreddit))
        .route(
            "/r/{subreddit}/comments/{post_id}",
            get(routes::post::get_post),
        );

    // Search routes
    let search_routes = Router::new()
        .route("/search", get(routes::search::search));

    // Discovery routes
    let discovery_routes = Router::new()
        .route("/discover", get(routes::subreddit_discovery::discover));

    // User routes
    let user_routes = Router::new()
        .route("/u/{username}", get(routes::user::get_user));

    // Auth routes
    let auth_routes = Router::new()
        .route("/auth/login", get(routes::auth::login_page))
        .route("/auth/start", get(routes::auth::login))
        .route("/auth/callback", get(routes::auth::callback))
        .route("/auth/logout", get(routes::auth::logout));

    // Settings
    let settings_routes = Router::new()
        .route("/settings", get(routes::settings::settings_page));

    // API routes
    let api_routes = Router::new()
        .route("/api/vote", post(routes::api::api_vote))
        .route("/api/comment", post(routes::api::api_comment))
        .route("/api/health", get(routes::api::api_health));

    // Assemble the router
    Router::new()
        .merge(static_router)
        .merge(media_routes)
        .merge(frontpage_routes)
        .merge(subreddit_routes)
        .merge(search_routes)
        .merge(discovery_routes)
        .merge(user_routes)
        .merge(auth_routes)
        .merge(settings_routes)
        .merge(api_routes)
        // Shared state
        .with_state(config.clone())
        .layer(Arc::new(reddit))
        .layer(Arc::new(media_proxy))
        .layer(Arc::new(cache_layer))
        // Middleware stack (outer → inner)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().gzip(true).br(true))
        .layer(csp_header())
        .layer(axum::middleware::map_response(security_headers))
        .layer(axum::middleware::from_fn(crate::middleware::cache::CacheLayer::cache_middleware))
}
