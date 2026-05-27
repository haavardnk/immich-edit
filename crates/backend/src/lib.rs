pub mod app;
pub mod config;
pub mod error;
pub mod immich;
pub mod lens_profile;
pub mod routes;
pub mod services;
pub mod state;
pub mod telemetry;

use std::time::Duration;
use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<()> {
    telemetry::init();

    let config = config::Config::load()?;
    let bind_socket = config.bind_socket;
    tracing::info!(config = ?config.redacted(), "loaded config");

    let state = state::AppState::new(config).await?;
    let queue = state.queue.clone();
    let app = app::router(state);

    tracing::info!("listening on {bind_socket}");
    let listener = TcpListener::bind(bind_socket).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("shutdown signal received; draining renders");
            queue.shutdown(Duration::from_secs(10)).await;
        })
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
