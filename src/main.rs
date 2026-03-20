mod config;
mod error;
mod extractors;
mod handlers;
mod models;
mod response;
mod router;
mod seed;
mod services;
mod state;
mod stores;
mod types;

use config::Config;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize tracing
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "cayopay_server=debug,tower_http=debug,axum::rejection=trace".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  // Load configuration
  let config = Config::init();

  // Connect to database
  tracing::info!("Connecting to database at {}...", config.database_url);
  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&config.database_url)
    .await
    .expect("Failed to connect to database");

  // Run migrations
  if config.database_migrations {
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
      .run(&pool)
      .await
      .expect("Failed to run migrations");
  }

  // Initialize application state
  let state = AppState::new(&config, pool);

  // Seed database
  seed::owner(&state).await?;
  seed::wallets(&state).await?;

  // Create router
  let app = router::router(state);

  // Start server
  let addr_str = config.server_addr();
  let addr: SocketAddr = addr_str.parse().expect("Invalid server address");
  tracing::info!("Server listening on http://{}", addr);

  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;

  Ok(())
}

async fn shutdown_signal() {
  let ctrl_c = async {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
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

  tracing::info!("signal received, starting graceful shutdown");
}
