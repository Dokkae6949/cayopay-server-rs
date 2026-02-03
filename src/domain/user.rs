use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::Actor;
use crate::domain::Role;
use crate::types::{Email, HashedPassword, Id};

#[derive(Debug, Clone, FromRow)]
pub struct User {
  pub id: Id<User>,
  pub actor_id: Id<Actor>,
  pub email: Email,
  pub password_hash: HashedPassword,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl User {
  pub fn new(
    actor_id: Id<Actor>,
    email: Email,
    password_hash: HashedPassword,
    first_name: impl Into<String>,
    last_name: impl Into<String>,
    role: Role,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: Id::new(),
      actor_id,
      email,
      first_name: first_name.into(),
      last_name: last_name.into(),
      password_hash,
      role: role,
      created_at: now,
      updated_at: now,
    }
  }
}
