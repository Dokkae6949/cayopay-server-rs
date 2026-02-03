use crate::error::AppResult;
use crate::{domain::Actor, domain::Guest, types::Id};
use sqlx::{Executor, PgConnection, Postgres};

pub struct GuestStore;

impl GuestStore {
  pub async fn save(executor: &mut PgConnection, guest: Guest) -> AppResult<()> {
    sqlx::query!(
      r#"
      INSERT INTO guests (id, actor_id)
      VALUES ($1, $2)
      "#,
      guest.id.into_inner(),
      guest.actor_id.into_inner(),
    )
    .execute(&mut *executor)
    .await
    .map_err(crate::error::AppError::Database)?;

    Ok(())
  }

  pub async fn find_by_actor_id<'c, E>(
    executor: E,
    actor_id: &Id<Actor>,
  ) -> AppResult<Option<Guest>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let guest = sqlx::query_as!(
      Guest,
      r#"
      SELECT id, actor_id
      FROM guests
      WHERE actor_id = $1
      "#,
      actor_id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(guest)
  }

  pub async fn list_all<'c, E>(executor: E) -> AppResult<Vec<Guest>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let guests = sqlx::query_as!(
      Guest,
      r#"
      SELECT id, actor_id
      FROM guests
      "#
    )
    .fetch_all(executor)
    .await?;

    Ok(guests)
  }

  pub async fn remove_by_id<'c, E>(executor: E, id: &Id<Guest>) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM guests
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .execute(executor)
    .await?;

    Ok(())
  }
}
