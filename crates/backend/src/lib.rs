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
use std::time::Duration;
use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<()> {
    telemetry::init();

    let config = config::Config::load()?;
    let bind_socket = config.bind_socket;
    tracing::info!(config = ?config.redacted(), "loaded config");

    let state = state::AppState::new(config).await?;
    let queue = state.queue.clone();
    let immich = state.immich.clone();

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let runner = services::job_runner::JobRunner::new(
        state.jobs.clone(),
        std::sync::Arc::new(services::export::BatchExecutor::new(state.clone())),
        state.config.render_max_concurrency,
    );
    let runner_handle = tokio::spawn(runner.run(shutdown_rx));

    tokio::spawn(async move {
        let status = immich::ImmichConnectionStatus::from_ping(immich.ping().await);
        if status.ok {
            return;
        }
        tracing::warn!(
            kind = status.kind,
            status_code = ?status.status_code,
            message = %status.message,
            "Immich ping failed at startup"
        );
    });
    let app = app::router(state);

    tracing::info!("listening on {bind_socket}");
    let listener = TcpListener::bind(bind_socket).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown_signal().await;
        tracing::info!("shutdown signal received; draining renders");
        let _ = shutdown_tx.send(true);
        queue.shutdown(Duration::from_secs(10)).await;
    })
    .await?;
    let _ = runner_handle.await;
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
