use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  domain::{Session, User},
  error::AppResult,
  stores::SessionStore,
  types::Id,
};

#[derive(Clone)]
pub struct SessionService {
  pool: PgPool,
  session_expiration_days: i64,
}

impl SessionService {
  pub fn new(pool: PgPool, session_expiration_days: i64) -> Self {
    Self {
      pool,
      session_expiration_days,
    }
  }

  pub async fn create_session(&self, user_id: Id<User>) -> AppResult<Session> {
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now()
      + Duration::try_days(self.session_expiration_days).unwrap_or(Duration::try_days(1).unwrap());
    let session = Session::new(token, user_id, expires_at);

    SessionStore::save(&self.pool, &session).await?;

    Ok(session)
  }

  pub async fn validate_session(&self, token: &str) -> AppResult<Option<Session>> {
    let session = SessionStore::find_by_token(&self.pool, token).await?;

    if let Some(ref s) = session {
      if s.is_expired() {
        SessionStore::delete_by_id(&self.pool, &s.id).await?;
        return Ok(None);
      }
    }

    Ok(session)
  }
}
