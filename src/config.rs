use std::env;
use std::net::SocketAddr;
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    /// Address the server listens on
    pub bind_addr: SocketAddr,

    /// Base URL for this instance (used for OAuth redirects and link generation)
    pub base_url: Url,

    /// Reddit OAuth2 client ID
    pub reddit_client_id: String,

    /// Reddit OAuth2 client secret
    pub reddit_client_secret: String,

    /// Cookie encryption key (32-byte hex)
    pub cookie_secret: String,

    /// Redis URL for caching (optional — uses in-memory moka cache if not set)
    pub redis_url: Option<String>,

    /// Proxy media through the server (default: true)
    pub proxy_media: bool,

    /// Allow NSFW content (default: true — instance admins can disable)
    pub allow_nsfw: bool,

    /// Require login to browse (default: false)
    pub require_login: bool,

    /// Max cache TTL in seconds (default: 300)
    pub cache_ttl_seconds: u64,

    /// Max request timeout to Reddit API in seconds (default: 30)
    pub upstream_timeout_seconds: u64,

    /// SOCKS5 proxy for Tor/I2P support (optional)
    pub socks5_proxy: Option<String>,

    /// Rate limit: max requests per minute per IP (default: 60)
    pub rate_limit_per_minute: u32,

    /// Quiet mode — suppress health check logs
    pub quiet: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Allow .env file in dev
        dotenvy::dotenv().ok();

        let bind_addr: SocketAddr = env::var("MURMUR_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".into())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid MURMUR_BIND_ADDR: {e}"))?;

        let base_url_str = env::var("MURMUR_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".into());
        let base_url = Url::parse(&base_url_str)
            .map_err(|e| anyhow::anyhow!("Invalid MURMUR_BASE_URL: {e}"))?;

        let reddit_client_id = env::var("MURMUR_REDDIT_CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("MURMUR_REDDIT_CLIENT_ID is required"))?;

        let reddit_client_secret = env::var("MURMUR_REDDIT_CLIENT_SECRET")
            .map_err(|_| anyhow::anyhow!("MURMUR_REDDIT_CLIENT_SECRET is required"))?;

        let cookie_secret = env::var("MURMUR_COOKIE_SECRET")
            .map_err(|_| anyhow::anyhow!("MURMUR_COOKIE_SECRET is required (32-byte hex string)"))?;

        Ok(Self {
            bind_addr,
            base_url,
            reddit_client_id,
            reddit_client_secret,
            cookie_secret,
            redis_url: env::var("MURMUR_REDIS_URL").ok(),
            proxy_media: env::var("MURMUR_PROXY_MEDIA")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(true),
            allow_nsfw: env::var("MURMUR_ALLOW_NSFW")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(true),
            require_login: env::var("MURMUR_REQUIRE_LOGIN")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            cache_ttl_seconds: env::var("MURMUR_CACHE_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            upstream_timeout_seconds: env::var("MURMUR_UPSTREAM_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            socks5_proxy: env::var("MURMUR_SOCKS5_PROXY").ok(),
            rate_limit_per_minute: env::var("MURMUR_RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            quiet: env::var("MURMUR_QUIET")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        })
    }
}
