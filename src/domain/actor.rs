use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::{Guest, User};
use crate::types::Id;

#[derive(Debug, Clone, FromRow)]
pub struct Actor {
  pub id: Id<Actor>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Actor {
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Id::new(),
      created_at: now,
      updated_at: now,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ActorWithDetails {
  pub actor: Actor,
  pub user: Option<User>,
  pub guest: Option<Guest>,
}
