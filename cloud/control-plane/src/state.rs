use std::sync::Arc;

use reqwest::Client;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::k8s::Provisioner;

pub struct AppState {
    pub config: Config,
    pub db: SqlitePool,
    pub k8s: Provisioner,
    pub http: Client,
}

pub type SharedState = Arc<AppState>;
