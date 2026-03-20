use sqlx::PgConnection;

use crate::models::{guest::GuestId, ActorId, Guest};
use crate::stores::models::guest::{GuestCreation, GuestRow, GuestUpdate};

pub struct GuestStore;

impl GuestStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &GuestCreation,
  ) -> Result<Guest, sqlx::Error> {
    let row = sqlx::query_as::<_, GuestRow>(
      r#"
      INSERT INTO guests (actor_id, email, verified)
      VALUES ($1, $2, $3)
      RETURNING id, actor_id, email, verified, created_at, updated_at
      "#,
    )
    .bind(creation.actor_id.into_inner())
    .bind(creation.email.expose())
    .bind(creation.verified)
    .fetch_one(conn)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id(
    conn: &mut PgConnection,
    id: &GuestId,
    update: &GuestUpdate,
  ) -> Result<Guest, sqlx::Error> {
    let row = sqlx::query_as::<_, GuestRow>(
      r#"
      UPDATE guests
      SET email = COALESCE($2, email),
          verified = COALESCE($3, verified)
      WHERE id = $1
      RETURNING id, actor_id, email, verified, created_at, updated_at
      "#,
    )
    .bind(id.into_inner())
    .bind(update.email.as_ref().map(|e| e.expose()))
    .bind(update.verified)
    .fetch_one(conn)
    .await?;

    Ok(row.into())
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &GuestId,
  ) -> Result<Option<Guest>, sqlx::Error> {
    let row = sqlx::query_as::<_, GuestRow>(
      r#"
      SELECT id, actor_id, email, verified, created_at, updated_at
      FROM guests
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_actor_id(
    conn: &mut PgConnection,
    actor_id: &ActorId,
  ) -> Result<Option<Guest>, sqlx::Error> {
    let row = sqlx::query_as::<_, GuestRow>(
      r#"
      SELECT id, actor_id, email, verified, created_at, updated_at
      FROM guests
      WHERE actor_id = $1
      "#,
    )
    .bind(actor_id.into_inner())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all(conn: &mut PgConnection) -> Result<Vec<Guest>, sqlx::Error> {
    let rows = sqlx::query_as::<_, GuestRow>(
      r#"
      SELECT id, actor_id, email, verified, created_at, updated_at
      FROM guests
      "#,
    )
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
