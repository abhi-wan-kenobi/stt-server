use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
pub struct Config {
    #[arg(long, env = "STT_HOST", default_value = "0.0.0.0")]
    pub host: String,
    #[arg(long, env = "STT_PORT", default_value_t = 8383)]
    pub port: u16,
    #[arg(long, env = "STT_MODEL_DIR", default_value = "./data/models")]
    pub model_dir: String,
    #[arg(long, env = "STT_MODEL", default_value = "whisper-large-v3-turbo-q5_0")]
    pub model: String,
    #[arg(long, env = "STT_REQUIRE_GPU", default_value_t = false)]
    pub require_gpu: bool,
    #[arg(long, env = "STT_TOKEN")]
    pub token: Option<String>,
    #[arg(long, env = "STT_HEALTH_MIN_RTF", default_value_t = 2.0)]
    pub health_min_rtf: f32,
    #[arg(long, env = "STT_HEALTH_INTERVAL_SECS", default_value_t = 300)]
    pub health_interval_secs: u64,
    #[arg(long, env = "STT_HEALTH_FAIL_STREAK", default_value_t = 2)]
    pub health_fail_streak: u32,
    #[arg(long, env = "STT_HEALTH_AUTORESTART", default_value_t = true)]
    pub health_autorestart: bool,
}