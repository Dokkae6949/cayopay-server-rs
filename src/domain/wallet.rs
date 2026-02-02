use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::Actor;
use crate::types::Id;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Wallet {
  pub id: Id<Wallet>,
  pub owner_actor_id: Id<Actor>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Wallet {
  pub fn new(owner_actor_id: Id<Actor>) -> Self {
    let now = Utc::now();
    Self {
      id: Id::new(),
      owner_actor_id,
      created_at: now,
      updated_at: now,
    }
  }
}
