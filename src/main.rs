use cayopay_server::{app, config::Config};
use domain::{wallet::WalletLabel, Role};
use sqlx::postgres::PgPoolOptions;
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

  // Seed database
  seed_owner(&config, &pool).await?;
  seed_wallets(&pool).await?;

  // Create app
  let smtp_config = config.smtp_config();
  let app = app(pool, smtp_config);

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

async fn seed_owner(config: &Config, pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
  use domain::RawPassword;
  use uuid::Uuid;
  
  // Check if owner exists
  let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
    .bind(config.owner_email.expose())
    .fetch_one(pool)
    .await?;
  
  if exists {
    tracing::debug!("Default owner user already exists");
    return Ok(());
  }
  
  // Create owner user (inline, no abstraction)
  let mut tx = pool.begin().await?;
  
  let actor_id: Uuid = sqlx::query_scalar("INSERT INTO actors DEFAULT VALUES RETURNING id")
    .fetch_one(&mut *tx)
    .await?;
  
  let hashed = config.owner_password.hash()?;
  
  sqlx::query("INSERT INTO users (actor_id, email, password_hash, first_name, last_name, role) VALUES ($1, $2, $3, $4, $5, $6)")
    .bind(actor_id)
    .bind(config.owner_email.expose())
    .bind(hashed.expose())
    .bind(&config.owner_first_name)
    .bind(&config.owner_last_name)
    .bind(Role::Owner.to_string())
    .execute(&mut *tx)
    .await?;
  
  sqlx::query("INSERT INTO wallets (owner_actor_id, allow_overdraft) VALUES ($1, false)")
    .bind(actor_id)
    .execute(&mut *tx)
    .await?;
  
  tx.commit().await?;
  
  tracing::info!("Seeded default owner user");
  Ok(())
}

async fn seed_wallets(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
  for label in WalletLabel::variants() {
    match sqlx::query("INSERT INTO wallets (label, allow_overdraft) VALUES ($1, true)")
      .bind(label.to_string())
      .execute(pool)
      .await
    {
      Ok(_) => tracing::info!("Seeded wallet with label {:?}", label),
      Err(sqlx::Error::Database(db_err))
        if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation =>
      {
        tracing::debug!("Wallet with label {:?} already exists", label);
      }
      Err(e) => {
        tracing::warn!("Failed to seed wallet with label {:?}: {}", label, e);
        return Err(Box::new(e));
      }
    }
  }

  Ok(())
}
