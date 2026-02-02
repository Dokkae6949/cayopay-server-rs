use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::bootstrap::seed;
use crate::config::Config;

pub async fn init_database(config: &Config) -> Result<PgPool, sqlx::Error> {
  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&config.database_url)
    .await?;

  if config.database_migrations {
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
  }

  Ok(pool)
}
