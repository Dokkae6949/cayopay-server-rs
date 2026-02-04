use chrono::{DateTime, Duration, Utc};
use domain::UserId;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct SessionRow {
  pub id: Uuid,
  pub user_id: Uuid,
  pub token: String,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct SessionCreation {
  pub user_id: UserId,
  pub token: String,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub expires_in: Duration,
}

impl From<SessionRow> for domain::Session {
  fn from(value: SessionRow) -> Self {
    Self {
      id: value.id.into(),
      user_id: value.user_id.into(),
      token: value.token,
      user_agent: value.user_agent,
      ip_address: value.ip_address,
      expires_in: value.expires_at - value.created_at,
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
