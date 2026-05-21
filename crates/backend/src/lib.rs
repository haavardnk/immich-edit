pub mod app;
pub mod config;
pub mod error;
pub mod state;
pub mod telemetry;

use std::net::SocketAddr;

use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<()> {
    telemetry::init();

    let config = config::Config::load();
    let state = state::AppState::new(config);
    let app = app::router(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    tracing::info!("listening on {addr}");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
