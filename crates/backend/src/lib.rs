pub mod app;
pub mod config;
pub mod error;
pub mod immich;
pub mod lens_profile;
pub mod routes;
pub mod services;
pub mod state;
pub mod telemetry;

use std::net::SocketAddr;

use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<()> {
    telemetry::init();

    let config = config::Config::load()?;
    let bind_addr = config.bind_addr.clone();
    tracing::info!(config = ?config.redacted(), "loaded config");

    let state = state::AppState::new(config).await?;
    let app = app::router(state);

    let addr: SocketAddr = bind_addr.parse()?;
    tracing::info!("listening on {addr}");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
