use anyhow::Result;
use clap::Parser;
use stt_server::config::Config;

/// Standalone GPU STT server (transcribe.cpp / ggml).
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(flatten)]
    config: Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stt_server=info,tower_http=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let cfg = cli.config;
    tracing::info!(?cfg, "stt-server starting");

    let app = stt_server::router::build(cfg.clone()).await?;
    let listener = tokio::net::TcpListener::bind((cfg.host.as_str(), cfg.port)).await?;
    tracing::info!("listening on http://{}:{}", cfg.host, cfg.port);
    axum::serve(listener, app).await?;
    Ok(())
}