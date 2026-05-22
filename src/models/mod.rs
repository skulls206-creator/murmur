pub mod post;
pub mod comment;
pub mod subreddit;
pub mod user;

pub use post::*;
pub use comment::*;
pub use subreddit::*;
pub use user::*;

/// Sort modes for feeds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Hot,
    New,
    Top,
    Rising,
    Controversial,
    Best,
}

impl SortMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortMode::Hot => "hot",
            SortMode::New => "new",
            SortMode::Top => "top",
            SortMode::Rising => "rising",
            SortMode::Controversial => "controversial",
            SortMode::Best => "best",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "new" => SortMode::New,
            "top" => SortMode::Top,
            "rising" => SortMode::Rising,
            "controversial" => SortMode::Controversial,
            "best" => SortMode::Best,
            _ => SortMode::Hot,
        }
    }
}

/// Time filters for top/controversial sorts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFilter {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

impl TimeFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFilter::Hour => "hour",
            TimeFilter::Day => "day",
            TimeFilter::Week => "week",
            TimeFilter::Month => "month",
            TimeFilter::Year => "year",
            TimeFilter::All => "all",
        }
    }
}

/// Feed view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedView {
    Card,
    Compact,
    Classic,
}

impl FeedView {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedView::Card => "card",
            FeedView::Compact => "compact",
            FeedView::Classic => "classic",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "compact" => FeedView::Compact,
            "classic" => FeedView::Classic,
            _ => FeedView::Card,
        }
    }
}

/// Pagination cursor (Reddit uses "after" / "before" fullnames)
#[derive(Debug, Clone)]
pub struct PageCursor {
    pub after: Option<String>,
    pub before: Option<String>,
    pub count: usize,
}
