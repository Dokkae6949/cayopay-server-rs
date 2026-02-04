use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use domain::{Actor, Email, Id, Role, User};

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
  pub id: Id<User>,
  pub actor_id: Id<Actor>,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      actor_id: user.actor_id,
      email: user.email,
      first_name: user.first_name,
      last_name: user.last_name,
      role: user.role,
      created_at: user.created_at,
      updated_at: user.updated_at,
    }
  }
}
