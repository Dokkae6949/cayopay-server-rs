use crate::models::{Invite, InviteId};
use crate::stores::models::invite::{InviteCreation, InviteRow, InviteUpdate};
use crate::types::Email;
use sqlx::PgConnection;

pub struct InviteStore;

impl InviteStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &InviteCreation,
  ) -> Result<Invite, sqlx::Error> {
    let row = sqlx::query_as::<_, InviteRow>(
      r#"
      INSERT INTO invites (invitor_user_id, email, token, role, expires_at)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      "#,
    )
    .bind(creation.invitor.into_inner())
    .bind(creation.email.expose())
    .bind(&creation.token)
    .bind(&creation.role)
    .bind(chrono::Utc::now() + creation.expires_in)
    .fetch_one(&mut *conn)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id(
    conn: &mut PgConnection,
    id: &InviteId,
    update: &InviteUpdate,
  ) -> Result<Option<Invite>, sqlx::Error> {
    let row = sqlx::query_as::<_, InviteRow>(
      r#"
      UPDATE invites
      SET status = COALESCE($2, status)
      WHERE id = $1
      RETURNING id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      "#,
    )
    .bind(id.into_inner())
    .bind(update.status.as_ref().map(ToString::to_string))
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn delete_by_id(conn: &mut PgConnection, id: &InviteId) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invites WHERE id = $1")
      .bind(id.into_inner())
      .execute(&mut *conn)
      .await?;

    Ok(())
  }

  pub async fn find_by_token(
    conn: &mut PgConnection,
    token: &str,
  ) -> Result<Option<Invite>, sqlx::Error> {
    let row = sqlx::query_as::<_, InviteRow>(
      r#"
      SELECT id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      FROM invites
      WHERE token = $1
      "#,
    )
    .bind(token)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_email(
    conn: &mut PgConnection,
    email: &Email,
  ) -> Result<Option<Invite>, sqlx::Error> {
    let row = sqlx::query_as::<_, InviteRow>(
      r#"
      SELECT id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      FROM invites
      WHERE email = $1
      "#,
    )
    .bind(email.expose())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all(conn: &mut PgConnection) -> Result<Vec<Invite>, sqlx::Error> {
    let rows = sqlx::query_as::<_, InviteRow>(
      r#"
      SELECT id, invitor_user_id, email, token, role, status, expires_at, created_at, updated_at
      FROM invites
      "#,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
