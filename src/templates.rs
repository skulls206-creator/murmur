/// Template filters and helper functions for Askama templates
use crate::proxy::media_proxy::encode_media_url;

/// Convert a Unix timestamp to an ISO 8601 string
pub fn timestamp_to_iso(timestamp: &f64) -> String {
    let dt = chrono::DateTime::from_timestamp(*timestamp as i64, 0)
        .unwrap_or_default();
    dt.to_rfc3339()
}

/// Convert a Unix timestamp to a human-readable relative time string
pub fn relative_time(timestamp: &f64) -> String {
    let now = chrono::Utc::now();
    let dt = chrono::DateTime::from_timestamp(*timestamp as i64, 0)
        .unwrap_or_default();
    let duration = now.signed_duration_since(dt);

    if duration.num_minutes() < 1 {
        "just now".into()
    } else if duration.num_minutes() < 60 {
        let m = duration.num_minutes();
        format!("{m}m ago")
    } else if duration.num_hours() < 24 {
        let h = duration.num_hours();
        format!("{h}h ago")
    } else if duration.num_days() < 7 {
        let d = duration.num_days();
        format!("{d}d ago")
    } else if duration.num_weeks() < 52 {
        let w = duration.num_weeks();
        format!("{w}w ago")
    } else {
        let y = duration.num_days() / 365;
        format!("{y}y ago")
    }
}

/// Register custom filters with Askama
pub fn register_filters() {
    // Filters are registered via Askama's `#[derive(Template)]` macro
    // See the template files for usage:
    //   {{ value | timestamp_to_iso }}
    //   {{ value | relative_time }}
    //   {{ value | encode_media_url }}
}
