mod config;
mod error;
mod routes;
mod state;

use std::net::SocketAddr;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::routes::create_router;
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

    // 4. Initialize Database Connection Pool
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("Connected to PostgreSQL database successfully");
            // Run automatic migrations
            match sqlx::migrate!("./migrations").run(&pool).await {
                Ok(_) => tracing::info!("Database migrations executed successfully"),
                Err(e) => tracing::error!("Failed to execute database migrations: {e}"),
            }
            Some(pool)
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "Could not connect to PostgreSQL immediately. Backend will run in degraded/standalone mode until DB is available."
            );
            None
        }
    };

    // 5. Initialize application state and router
    let state = AppState::new(pool, config.clone());
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
