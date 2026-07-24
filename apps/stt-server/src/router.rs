use crate::config::Config;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub async fn build(cfg: Config) -> anyhow::Result<Router> {
    let state = Arc::new(crate::state::AppState::new(cfg.clone()).await?);
    let cors = CorsLayer::new(); // TODO(port): Tauri-webview-origin allowlist.

    let app = Router::new()
        .route("/", axum::routing::get(landing))
        .route("/dashboard", axum::routing::get(dashboard))
        .route("/health", axum::routing::get(health))
        .route("/api/status", axum::routing::get(api_status))
        .route("/api/models", axum::routing::get(api_models))
        .route("/api/metrics", axum::routing::get(api_metrics))
        .route("/api/sessions", axum::routing::get(api_sessions))
        .route("/api/models/{id}/download", axum::routing::post(model_download))
        .route("/api/models/{id}/progress", axum::routing::get(model_progress))
        .route("/api/models/{id}/cancel", axum::routing::post(model_cancel))
        .route("/api/models/{id}/activate", axum::routing::post(model_activate))
        .route("/api/models/{id}", axum::routing::delete(model_delete))
        .route("/v1/listen", axum::routing::post(listen_batch).get(listen_ws))
        .route("/api/tailscale", axum::routing::get(tailscale_status).post(tailscale_toggle))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(app)
}

async fn landing() -> &'static str { crate::assets::INDEX_HTML }
async fn dashboard() -> &'static str { crate::assets::DASHBOARD_HTML }
async fn health() -> &'static str { "ok" }
async fn api_status() -> &'static str { "{}" }
async fn api_models() -> &'static str { "[]" }
async fn api_metrics() -> &'static str { "# HELP stt_server_up 1 if up\n# TYPE stt_server_up gauge\nstt_server_up 1\n" }
async fn api_sessions() -> &'static str { "{}" }
async fn model_download() -> &'static str { "{}" }
async fn model_progress() -> &'static str { "{}" }
async fn model_cancel() -> &'static str { "{}" }
async fn model_activate() -> &'static str { "{}" }
async fn model_delete() -> &'static str { "{}" }
async fn listen_batch() -> &'static str { "{}" }
async fn listen_ws() -> &'static str { "ws" }
async fn tailscale_status() -> &'static str { "{}" }
async fn tailscale_toggle() -> &'static str { "{}" }