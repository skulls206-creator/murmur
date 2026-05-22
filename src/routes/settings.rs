use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};

use crate::config::Config;
use crate::models::UserSession;

#[derive(Template)]
#[template(path = "pages/settings.html")]
pub struct SettingsTemplate {
    pub config: Arc<Config>,
    pub user: Option<UserSession>,
}

pub async fn settings_page(
    State(config): State<Arc<Config>>,
) -> impl IntoResponse {
    let template = SettingsTemplate {
        config,
        user: None,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template error".into()),
    )
}
