use crate::config::Config;
use std::sync::Arc;

pub struct AppState {
    pub cfg: Config,
}

impl AppState {
    pub async fn new(cfg: Config) -> anyhow::Result<Self> {
        Ok(Self { cfg })
    }
}

pub type SharedState = Arc<AppState>;