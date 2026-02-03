use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::{Role, User};
use crate::types::{Email, Id};

#[derive(Debug, Clone, FromRow)]
pub struct Invite {
  pub id: Id<Invite>,
  pub created_by: Id<User>,
  pub email: Email,
  pub token: String,
  pub role: Role,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
