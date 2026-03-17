use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use domain::{Session, User, UserId};
use infra::stores::{models::SessionCreation, SessionStore, UserStore};

pub async fn create(pool: &PgPool, user_id: UserId, expiration_days: i64) -> AppResult<Session> {
  let token = Uuid::new_v4().to_string();

  let session = SessionStore::create(
    pool,
    &SessionCreation {
      user_id: user_id.into(),
      token,
      user_agent: None,
      ip_address: None,
      expires_in: Duration::days(expiration_days),
    },
  )
  .await?;

  Ok(session)
}

pub async fn get(pool: &PgPool, token: &str) -> AppResult<Option<Session>> {
  let session = SessionStore::find_by_token(pool, token).await?;

  if let Some(ref s) = session {
    if s.is_expired() {
      SessionStore::delete_by_token(pool, token).await?;
      return Ok(None);
    }
  }

  Ok(session)
}

/// Validates a session token and returns the owning user, or `None` if the
/// session is missing or expired.
pub async fn authenticate(pool: &PgPool, token: &str) -> AppResult<Option<User>> {
  let Some(session) = get(pool, token).await? else {
    return Ok(None);
  };
  Ok(UserStore::find_by_id(pool, &session.user_id).await?)
}

pub async fn end(pool: &PgPool, token: &str) -> AppResult<()> {
  SessionStore::delete_by_token(pool, token).await?;
  Ok(())
}
