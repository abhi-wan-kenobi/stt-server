//! Embedded HTML assets (no Node build step — `include_str!`).

pub const INDEX_HTML: &str = include_str!("../assets/index.html");
pub const DASHBOARD_HTML: &str = include_str!("../assets/dashboard.html");