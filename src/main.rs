mod config;
mod error;
mod middleware;
mod models;
mod proxy;
mod router;
mod routes;
mod templates;

use std::sync::Arc;

use tracing::info;
use tracing_subscriber::EnvFilter;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,murmur=debug")),
        )
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    let bind_addr = config.bind_addr;

    if !config.quiet {
        info!(
            "Murmur v{} starting on {}",
            env!("CARGO_PKG_VERSION"),
            bind_addr
        );
        info!("Base URL: {}", config.base_url);
        info!("Media proxy: {}", if config.proxy_media { "enabled" } else { "disabled" });
        info!("NSFW content: {}", if config.allow_nsfw { "allowed" } else { "blocked" });
        info!("Login required: {}", if config.require_login { "yes" } else { "no" });
        info!("Cache TTL: {}s", config.cache_ttl_seconds);
        if let Some(socks5) = &config.socks5_proxy {
            info!("SOCKS5 proxy: {socks5}");
        }
    }

    // Build the application
    let app = router::build_router(config);

    // Start the server
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    if !Config::from_env().unwrap().quiet {
        info!("Server ready → http://{bind_addr}");
    }

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;

    Ok(())
}
