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
    let instance = state.instance.clone();
    let ping_timeout = state.config.original_timeout_secs;

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let runner = services::job_runner::JobRunner::new(
        state.jobs.clone(),
        std::sync::Arc::new(services::export::BatchExecutor::new(state.clone())),
        state.config.render_max_concurrency,
    );
    let runner_handle = tokio::spawn(runner.run(shutdown_rx));

    let cleanup_auth = state.auth.clone();
    let mut cleanup_shutdown = shutdown_tx.subscribe();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(6 * 3600));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = cleanup_auth.cleanup_expired().await {
                        tracing::warn!(error = %err, "expired session cleanup failed");
                    }
                }
                _ = cleanup_shutdown.changed() => break,
            }
        }
    });

    #[cfg(feature = "segment")]
    {
        let segment = state.segment.clone();
        let mut segment_shutdown = shutdown_tx.subscribe();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            loop {
                tokio::select! {
                    _ = interval.tick() => segment.release_idle().await,
                    _ = segment_shutdown.changed() => break,
                }
            }
        });
    }

    tokio::spawn(async move {
        let Ok(cfg) = instance.get().await else {
            return;
        };
        let Some(url) = cfg.immich_url else {
            tracing::info!("instance not configured; complete setup in the browser");
            return;
        };
        let Ok(base) = url::Url::parse(&url) else {
            return;
        };
        let Ok(client) = immich::ImmichClient::with_auth(
            base,
            immich::client::ImmichAuth::ApiKey(String::new()),
            Duration::from_secs(ping_timeout),
        ) else {
            return;
        };
        let status = immich::ImmichConnectionStatus::from_ping(client.ping().await);
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
