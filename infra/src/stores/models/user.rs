use chrono::{DateTime, Utc};
use domain::{ActorId, Email, HashedPassword, Role, User};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct UserRow {
  pub id: Uuid,
  pub actor_id: Uuid,
  pub email: String,
  pub password_hash: String,
  pub first_name: String,
  pub last_name: String,
  pub role: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct UserCreation {
  pub actor_id: ActorId,
  pub email: Email,
  pub password: HashedPassword,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
}

#[derive(Clone)]
pub struct UserUpdate {
  pub email: Option<Email>,
  pub password: Option<HashedPassword>,
  pub first_name: Option<String>,
  pub last_name: Option<String>,
  pub role: Option<Role>,
}

impl From<UserRow> for User {
  fn from(value: UserRow) -> Self {
    Self {
      id: value.id.into(),
      actor_id: value.actor_id.into(),
      email: value.email.into(),
      password: value.password_hash.into(),
      first_name: value.first_name,
      last_name: value.last_name,
      role: value.role.into(),
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
