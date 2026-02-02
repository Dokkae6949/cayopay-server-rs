#![allow(unused)]

use std::net::SocketAddr;

mod api;
mod bootstrap;
mod config;
mod domain;
mod error;
mod services;
mod state;
mod stores;
mod types;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::init();

  bootstrap::init_tracing();
  tracing::info!("Tracing initialized");

  let pool = bootstrap::init_database(&config).await?;
  tracing::info!("Database initialized");

  let state = AppState::new(&config, pool);
  tracing::info!("App state initialized");

  bootstrap::seed_database(&state).await?;
  tracing::info!("Database seeded");

  let app = api::router(state);

  let addr: SocketAddr = config.server_addr().parse()?;
  tracing::info!("Server listening on {}", addr);

  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}
