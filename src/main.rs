use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod app_state;
mod config;
mod domain;
mod error;
mod services;
mod stores;
mod types;

use crate::app_state::AppState;
use crate::config::Config;
use crate::stores::UserStore;
use crate::types::{Email, RawPassword};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::init();

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "cayopay_server=debug,tower_http=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  tracing::info!("Starting server...");

  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&config.database_url)
    .await?;

  tracing::info!("Running database migrations...");
  sqlx::migrate!("./migrations").run(&pool).await?;

  let state = AppState::new(&config, pool.clone());

  let email = Email::new("a@b.c");
  if UserStore::find_by_email(&pool, &email).await?.is_none() {
    tracing::info!("Seeding default admin user (a@b.c / password)");
    state
      .auth_service
      .register(
        email,
        RawPassword::new("password"),
        "Nimda".to_string(),
        "Admin".to_string(),
      )
      .await?;
  }

  let app = api::router(state);

  let addr: SocketAddr = config.server_addr().parse()?;
  tracing::info!("listening on {}", addr);

  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}
