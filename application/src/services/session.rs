use chrono::Duration;
use infra::stores::{models::SessionCreation, SessionStore};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use domain::{Session, UserId};

#[derive(Clone)]
pub struct SessionService {
  pool: PgPool,
  expiration_days: i64,
}

impl SessionService {
  pub fn new(pool: PgPool, expiration_days: i64) -> Self {
    Self {
      pool,
      expiration_days,
    }
  }

  pub async fn create_session(&self, user_id: UserId) -> AppResult<Session> {
    let token = Uuid::new_v4().to_string();

    let new_session = SessionCreation {
      user_id: user_id.into(),
      token,
      user_agent: None,
      ip_address: None,
      expires_in: Duration::days(self.expiration_days),
    };

    let session = SessionStore::create(&self.pool, &new_session).await?;

    Ok(session)
  }

  pub async fn get_session(&self, token: &str) -> AppResult<Option<Session>> {
    let session = SessionStore::find_by_token(&self.pool, token).await?;

    if let Some(ref s) = session {
      if s.is_expired() {
        SessionStore::delete_by_token(&self.pool, token).await?;
        return Ok(None);
      }
    }

    Ok(session)
  }

  pub async fn end_session(&self, token: &str) -> AppResult<()> {
    SessionStore::delete_by_token(&self.pool, token).await?;
    Ok(())
  }
}
