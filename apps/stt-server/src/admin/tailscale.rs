//! `/api/tailscale` — landing-page toggle for the built-in Tailscale sidecar.
//!
//! The server itself is plain HTTP. For non-loopback exposure (clients that
//! require HTTPS/wss on custom servers), ship a Tailscale sidecar that shares
//! the network namespace and terminates HTTPS via Tailscale Serve on a
//! `*.ts.net` name — tailnet-only by default. The landing-page toggle walks
//! the operator through auth-key + hostname + Serve/Funnel setup.
//!
//! v0.1 scope: `tailscale_toggle` writes `TS_AUTHKEY` + `TS_HOSTNAME` to the
//! configured env file (default `docker/.env` next to the server binary) and
//! returns the resulting ts.net URL. The operator then runs
//! `docker compose up -d` to apply. Full Docker-socket restart is a v0.2
//! roadmap item (`docs/roadmap.md`).

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Request body for `POST /api/tailscale`.
#[derive(Debug, Deserialize)]
pub struct TailscaleToggleRequest {
    /// Tailscale auth key (preauth, ephemeral, tagged — see docker/.env.example).
    pub ts_authkey: String,
    /// Tailnet hostname (e.g. `stt`). Final URL = `https://<hostname>.<tailnet>.ts.net/v1`.
    pub hostname: String,
    /// false = Tailscale Serve (tailnet-only, default). true = Tailscale Funnel (public internet).
    #[serde(default)]
    pub funnel: bool,
}

/// Response for both `GET` and `POST /api/tailscale`.
#[derive(Debug, Serialize)]
pub struct TailscaleStatus {
    /// Whether the env file has a non-empty `TS_AUTHKEY` (i.e. toggle is "on").
    pub exposed: bool,
    /// The ts.net URL the server will be reachable at once `docker compose up -d`
    /// is run, or `null` if `hostname` isn't set. Always carries the `/v1` suffix
    /// (the desktop client appends `/listen` to it).
    pub url: Option<String>,
    /// Whether Funnel (public internet) is enabled in the env file.
    pub funnel: bool,
    /// v0.1: the next manual step. v0.2 will auto-restart via Docker socket.
    pub next_step: String,
}

/// `GET /api/tailscale` — current exposure state, read from the env file.
pub async fn tailscale_status(State(state): State<Arc<AppState>>) -> Response {
    let env_path = state.config.tailscale_env_path();
    let (ts_authkey, hostname, funnel) = read_env(&env_path);
    let exposed = !ts_authkey.trim().is_empty();
    let url = hostname
        .filter(|h| !h.trim().is_empty())
        .map(|h| format!("https://{h}.ts.net/v1"));
    let status = TailscaleStatus {
        exposed,
        url,
        funnel,
        next_step: if exposed {
            "Run: docker compose -f docker/docker-compose.tailscale.yml up -d".to_string()
        } else {
            "POST /api/tailscale with {ts_authkey, hostname, funnel} to enable".to_string()
        },
    };
    (StatusCode::OK, Json(status)).into_response()
}

/// `POST /api/tailscale` — write the auth key + hostname (+ Funnel flag) to the
/// env file so the next `docker compose up -d` picks them up. Bearer-token-gated
/// via the protected sub-router in `admin::router`.
pub async fn tailscale_toggle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TailscaleToggleRequest>,
) -> Response {
    if req.ts_authkey.trim().is_empty() || req.hostname.trim().is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "ts_authkey and hostname are required");
    }
    let env_path = state.config.tailscale_env_path();
    if let Err(e) = write_env(&env_path, &req.ts_authkey, &req.hostname, req.funnel) {
        tracing::error!(path = %env_path.display(), error = %e, "tailscale_toggle: write failed");
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write env file: {e}"),
        );
    }
    tracing::info!(
        hostname = %req.hostname,
        funnel = req.funnel,
        path = %env_path.display(),
        "tailscale_toggle: env written; operator must run docker compose up -d"
    );
    let url = format!("https://{}.ts.net/v1", req.hostname);
    let status = TailscaleStatus {
        exposed: true,
        url: Some(url.clone()),
        funnel: req.funnel,
        next_step: "Run: docker compose -f docker/docker-compose.tailscale.yml up -d".to_string(),
    };
    (StatusCode::OK, Json(status)).into_response()
}

fn read_env(path: &PathBuf) -> (String, Option<String>, bool) {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return (String::new(), None, false);
    };
    let mut ts_authkey = String::new();
    let mut hostname: Option<String> = None;
    let mut funnel = false;
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let (k, v) = (k.trim(), v.trim().trim_matches('"'));
            match k {
                "TS_AUTHKEY" => ts_authkey = v.to_string(),
                "TS_HOSTNAME" => hostname = Some(v.to_string()),
                "TS_FUNNEL" => funnel = v.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }
    }
    (ts_authkey, hostname, funnel)
}

fn write_env(path: &PathBuf, ts_authkey: &str, hostname: &str, funnel: bool) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = format!(
        "# Written by stt-server POST /api/tailscale at {}\n\
         TS_AUTHKEY={}\n\
         TS_HOSTNAME={}\n\
         TS_FUNNEL={}\n",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        ts_authkey,
        hostname,
        if funnel { "true" } else { "false" }
    );
    std::fs::write(path, contents)
}

fn error_response(status: StatusCode, detail: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": "tailscale_config_failed",
            "detail": detail,
        })),
    )
        .into_response()
}
