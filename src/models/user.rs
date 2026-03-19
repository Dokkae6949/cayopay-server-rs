use chrono::{DateTime, Utc};

use crate::models::actor::ActorId;
use crate::types::{Email, HashedPassword, Id};

pub type UserId = Id<User>;

#[derive(Debug, Clone)]
pub struct User {
  pub id: UserId,
  pub actor_id: ActorId,
  pub email: Email,
  pub password: HashedPassword,
  pub first_name: String,
  pub last_name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
