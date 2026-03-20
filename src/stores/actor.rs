use crate::models::actor::ActorId;
use sqlx::{PgConnection, Row};

pub struct ActorStore;

impl ActorStore {
  pub async fn create(conn: &mut PgConnection) -> Result<ActorId, sqlx::Error> {
    let row = sqlx::query("INSERT INTO actors DEFAULT VALUES RETURNING id")
      .fetch_one(conn)
      .await?;

    Ok(row.try_get::<uuid::Uuid, _>("id")?.into())
  }
}
