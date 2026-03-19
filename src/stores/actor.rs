use crate::models::actor::ActorId;
use sqlx::{Executor, Postgres, Row};

pub struct ActorStore;

impl ActorStore {
  pub async fn create<'c, E>(executor: E) -> Result<ActorId, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query("INSERT INTO actors DEFAULT VALUES RETURNING id")
      .fetch_one(executor)
      .await?;

    Ok(row.try_get::<uuid::Uuid, _>("id")?.into())
  }
}
