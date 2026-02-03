use sqlx::{Executor, Postgres};

use crate::{
  domain::Invite,
  error::AppResult,
  types::{Email, Id},
};

pub struct InviteStore;

impl InviteStore {
  pub async fn save<'c, E>(executor: E, invite: &Invite) -> AppResult<Invite>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let result = sqlx::query_as!(
      Invite,
      r#"
      INSERT INTO invites (id, created_by, email, token, role, expires_at, created_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      RETURNING id, created_by, email, token, role, expires_at, created_at
      "#,
      invite.id.into_inner(),
      invite.created_by.into_inner(),
      invite.email.expose(),
      invite.token,
      invite.role.to_string(),
      invite.expires_at,
      invite.created_at
    )
    .fetch_one(executor)
    .await;

    match result {
      Ok(invite) => Ok(invite),
      Err(e) => Err(match &e {
        sqlx::Error::Database(db_err) => match db_err.kind() {
          sqlx::error::ErrorKind::UniqueViolation => crate::error::AppError::InviteAlreadySent,
          _ => crate::error::AppError::Database(e),
        },
        _ => crate::error::AppError::Database(e),
      }),
    }
  }

  pub async fn find_by_token<'c, E>(executor: E, token: &str) -> AppResult<Option<Invite>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let invite = sqlx::query_as!(
      Invite,
      r#"
      SELECT id, created_by, email, token, role, expires_at, created_at
      FROM invites
      WHERE token = $1
      "#,
      token
    )
    .fetch_optional(executor)
    .await?;

    Ok(invite)
  }

  pub async fn delete_by_id<'c, E>(executor: E, id: &Id<Invite>) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM invites
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn delete_expired_by_email<'c, E>(
    executor: E,
    email: &Email,
    now: chrono::DateTime<chrono::Utc>,
  ) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM invites
      WHERE email = $1 AND expires_at < $2
      "#,
      email.expose(),
      now
    )
    .execute(executor)
    .await?;

    Ok(())
  }
}
