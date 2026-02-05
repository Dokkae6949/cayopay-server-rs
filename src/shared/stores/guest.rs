use sqlx::{Executor, Postgres};

use crate::stores::models::guest::{GuestCreation, GuestRow, GuestUpdate};
use domain::{guest::GuestId, ActorId, Guest};

pub struct GuestStore;

impl GuestStore {
  pub async fn create<'c, E>(executor: E, creation: &GuestCreation) -> Result<Guest, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      GuestRow,
      r#"
      INSERT INTO guests (actor_id, email, verified)
      VALUES ($1, $2, $3)
      RETURNING id, actor_id, email, verified, created_at, updated_at
      "#,
      creation.actor_id.into_inner(),
      creation.email.expose(),
      creation.verified,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id<'c, E>(
    executor: E,
    id: &GuestId,
    update: &GuestUpdate,
  ) -> Result<Guest, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      GuestRow,
      r#"
      UPDATE guests
      SET email = COALESCE($2, email),
          verified = COALESCE($3, verified)
      WHERE id = $1
      RETURNING id, actor_id, email, verified, created_at, updated_at
      "#,
      id.into_inner(),
      update.email.as_ref().map(|e| e.expose()),
      update.verified,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &GuestId) -> Result<Option<Guest>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      GuestRow,
      r#"
      SELECT id, actor_id, email, verified, created_at, updated_at
      FROM guests
      WHERE id = $1
      "#,
      id.into_inner(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_actor_id<'c, E>(
    executor: E,
    actor_id: &ActorId,
  ) -> Result<Option<Guest>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      GuestRow,
      r#"
      SELECT id, actor_id, email, verified, created_at, updated_at
      FROM guests
      WHERE actor_id = $1
      "#,
      actor_id.into_inner(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all<'c, E>(executor: E) -> Result<Vec<Guest>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      GuestRow,
      r#"
      SELECT id, actor_id, email, verified, created_at, updated_at
      FROM guests
      "#
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
