use sqlx::{Executor, Postgres};

use crate::domain::Session;
use crate::error::AppResult;
use crate::types::Id;

pub struct SessionStore;

impl SessionStore {
  pub async fn save<'c, E>(executor: E, session: &Session) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      INSERT INTO sessions (id, user_id, token, expires_at, created_at)
      VALUES ($1, $2, $3, $4, $5)
      "#,
      session.id.into_inner(),
      session.user_id.into_inner(),
      session.token,
      session.expires_at,
      session.created_at
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_token<'c, E>(executor: E, token: &str) -> AppResult<Option<Session>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let session = sqlx::query_as!(
      Session,
      r#"
      SELECT id, token, user_id, expires_at, created_at
      FROM sessions
      WHERE token = $1
      "#,
      token
    )
    .fetch_optional(executor)
    .await?;

    Ok(session)
  }

  pub async fn delete_by_id<'c, E>(executor: E, id: &Id<Session>) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM sessions
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .execute(executor)
    .await?;

    Ok(())
  }
}
