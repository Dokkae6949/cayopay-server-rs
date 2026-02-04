use domain::{Email, Invite, InviteId};
use sqlx::{Executor, Postgres};

use crate::stores::models::invite::{InviteCreation, InviteRow, InviteUpdate};

pub struct InviteStore;

impl InviteStore {
  pub async fn create<'c, E>(executor: E, creation: &InviteCreation) -> Result<Invite, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      InviteRow,
      r#"
      INSERT INTO invites (invitor_user_id, email, token, role, expires_at)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      "#,
      creation.invitor.into_inner(),
      creation.email.expose(),
      creation.token,
      creation.role.to_string(),
      chrono::Utc::now() + creation.expires_in,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id<'c, E>(
    executor: E,
    id: &InviteId,
    update: &InviteUpdate,
  ) -> Result<Option<Invite>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      InviteRow,
      r#"
      UPDATE invites
      SET status = COALESCE($2, status)
      WHERE id = $1
      RETURNING id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      "#,
      id.into_inner(),
      update.status.as_ref().map(ToString::to_string)
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn delete_by_id<'c, E>(executor: E, id: &InviteId) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM invites
      WHERE id = $1
      "#,
      id.into_inner(),
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_token<'c, E>(executor: E, token: &str) -> Result<Option<Invite>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      InviteRow,
      r#"
      SELECT id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      FROM invites
      WHERE token = $1
      "#,
      token,
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_email<'c, E>(
    executor: E,
    email: &Email,
  ) -> Result<Option<Invite>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      InviteRow,
      r#"
      SELECT id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      FROM invites
      WHERE email = $1
      "#,
      email.expose(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn get_all<'c, E>(executor: E) -> Result<Vec<Invite>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      InviteRow,
      r#"
      SELECT id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      FROM invites
      "#
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
