use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind: String,
    pub admin_token: String,
    pub db_path: PathBuf,
    pub engine_image: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            bind: std::env::var("HOUSTON_CP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            admin_token: std::env::var("HOUSTON_CP_ADMIN_TOKEN")
                .unwrap_or_else(|_| "dev-admin-token-change-me".into()),
            db_path: std::env::var("HOUSTON_CP_DB_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/data/control-plane.db")),
            engine_image: std::env::var("HOUSTON_ENGINE_IMAGE")
                .unwrap_or_else(|_| "houston/engine:dev".into()),
        })
    }
}
