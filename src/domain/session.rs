use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::User;
use crate::types::Id;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Session {
  pub id: Id<Session>,
  pub user_id: Id<User>,
  pub token: String,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Session {
  pub fn new(token: String, user_id: Id<User>, expires_at: DateTime<Utc>) -> Self {
    Self {
      id: Id::new(),
      token,
      user_id,
      expires_at,
      created_at: Utc::now(),
    }
  }

  pub fn is_expired(&self) -> bool {
    self.expires_at < Utc::now()
  }
}
