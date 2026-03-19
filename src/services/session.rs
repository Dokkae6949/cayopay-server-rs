use sqlx::PgPool;

use crate::error::AppResult;
use crate::models::User;
use crate::stores::{SessionStore, UserStore};

/// Validates a session token and returns the owning user, or `None` if the
/// session is missing or expired.
pub async fn authenticate(pool: &PgPool, token: &str) -> AppResult<Option<User>> {
  let Some(session) = SessionStore::find_by_token(pool, token).await? else {
    return Ok(None);
  };

  if session.is_expired() {
    SessionStore::delete_by_token(pool, token).await?;
    return Ok(None);
  }

  Ok(UserStore::find_by_id(pool, &session.user_id).await?)
}
