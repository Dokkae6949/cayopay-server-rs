use chrono::{DateTime, Duration, Utc};

use crate::{Id, UserId};

pub type SessionId = Id<Session>;

#[derive(Debug, Clone)]
pub struct Session {
  pub id: SessionId,
  pub user_id: UserId,
  pub token: String,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub expires_in: Duration,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

impl Session {
  pub fn is_expired(&self) -> bool {
    Utc::now() > self.created_at + self.expires_in
  }
}
