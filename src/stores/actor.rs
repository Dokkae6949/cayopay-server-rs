use sqlx::{Executor, Postgres};

use crate::domain::Actor;
use crate::error::AppResult;
use crate::types::Id;

pub struct ActorStore;

impl ActorStore {
  pub async fn save<'c, E>(executor: E, actor: &Actor) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
            INSERT INTO actors (id, created_at, updated_at)
            VALUES ($1, $2, $3)
            "#,
      actor.id.into_inner(),
      actor.created_at,
      actor.updated_at
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &Id<Actor>) -> AppResult<Option<Actor>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let actor = sqlx::query_as!(
      Actor,
      r#"
        SELECT id, created_at, updated_at
        FROM actors
        WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(actor)
  }
}
