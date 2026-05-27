pub mod app;
pub mod config;
pub mod error;
pub mod immich;
pub mod lens_profile;
pub mod routes;
pub mod services;
pub mod state;
pub mod telemetry;

use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<()> {
    telemetry::init();

    let config = config::Config::load()?;
    let bind_socket = config.bind_socket;
    tracing::info!(config = ?config.redacted(), "loaded config");

    let state = state::AppState::new(config).await?;
    let app = app::router(state);

    tracing::info!("listening on {bind_socket}");
    let listener = TcpListener::bind(bind_socket).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
