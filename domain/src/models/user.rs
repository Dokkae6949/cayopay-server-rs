use chrono::{DateTime, Utc};

use crate::{actor::ActorId, Email, HashedPassword, Id, Role};

pub type UserId = Id<User>;

#[derive(Debug, Clone)]
pub struct User {
  pub id: UserId,
  pub actor_id: ActorId,
  pub email: Email,
  pub password: HashedPassword,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
