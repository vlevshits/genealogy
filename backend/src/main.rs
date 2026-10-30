mod config;
mod error;
pub mod models;
mod routes;
pub mod services;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::routes::create_router;
use crate::services::{
    auth::AuthService, backup::BackupService, db_init::connect_and_migrate,
    scheduler::spawn_background_scheduler,
};
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load environment variables from .env
    let _ = dotenvy::dotenv();

    // 2. Load typed application configuration
    let config = AppConfig::from_env().map_err(|e| format!("Configuration error: {e}"))?;

    // 3. Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.rust_log.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        app_name = "genealogy-backend",
        version = env!("CARGO_PKG_VERSION"),
        env = %config.app_env,
        "Starting Family Tree backend service"
    );

    // 4. Connect to PostgreSQL with exponential retry loop, advisory-locked migrations,
    //    and automatic pre-upgrade backup when pending migrations exist.
    let backup_service = BackupService::new(Arc::new(config.clone()));
    let pool = connect_and_migrate(&config, &backup_service)
        .await
        .map_err(|e| format!("Database initialization error: {e}"))?;

    // 5. Initialize application state, background scheduler, and router
    let state = AppState::new_empty(pool, config.clone());
    if let Err(e) = AuthService::hydrate_from_db(&state).await {
        tracing::error!("Failed to hydrate auth state from database: {e}");
    }
    spawn_background_scheduler(state.clone());
    let app = create_router(state);

    // 6. Bind listener and run Axum server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Backend server gracefully shut down");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
