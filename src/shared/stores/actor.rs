use domain::actor::ActorId;
use sqlx::{Executor, Postgres};

pub struct ActorStore;

impl ActorStore {
  pub async fn create<'c, E>(executor: E) -> Result<ActorId, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query!(
      r#"
      INSERT INTO actors DEFAULT VALUES
      RETURNING id
      "#
    )
    .fetch_one(executor)
    .await?;

    Ok(row.id.into())
  }
}
