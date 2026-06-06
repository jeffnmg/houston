use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod auth;
mod config;
mod db;
mod error;
mod k8s;
mod proxy;
mod routes;
mod state;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let db = db::init(&config.db_path).await?;
    let k8s = k8s::Provisioner::new(config.engine_image.clone()).await?;

    let state = Arc::new(AppState {
        config: config.clone(),
        db,
        k8s,
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("build reqwest client")?,
    });

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = config.bind.parse().context("parse HOUSTON_CP_BIND")?;
    tracing::info!(%addr, "houston control plane listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
