use domain::{Session, UserId};
use sqlx::{Executor, Postgres};

use crate::stores::models::session::{SessionCreation, SessionRow};

pub struct SessionStore;

impl SessionStore {
  pub async fn create<'c, E>(
    executor: E,
    creation: &SessionCreation,
  ) -> Result<Session, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      SessionRow,
      r#"
      INSERT INTO sessions (user_id, token, user_agent, ip_address, expires_at)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, user_id, token, user_agent, ip_address, expires_at, created_at, updated_at
      "#,
      creation.user_id.into_inner(),
      creation.token,
      creation.user_agent,
      creation.ip_address,
      chrono::Utc::now() + creation.expires_in,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn delete_by_token<'c, E>(executor: E, token: &str) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM sessions
      WHERE token = $1
      "#,
      token,
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_token<'c, E>(
    executor: E,
    token: &str,
  ) -> Result<Option<Session>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      SessionRow,
      r#"
      SELECT id, user_id, token, user_agent, ip_address, expires_at, created_at, updated_at
      FROM sessions
      WHERE token = $1
      "#,
      token,
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_user_id<'c, E>(
    executor: E,
    user_id: &UserId,
  ) -> Result<Vec<Session>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      SessionRow,
      r#"
      SELECT id, user_id, token, user_agent, ip_address, expires_at, created_at, updated_at
      FROM sessions
      WHERE user_id = $1
      "#,
      user_id.into_inner(),
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
