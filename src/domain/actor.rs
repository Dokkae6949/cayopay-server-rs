use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::types::Id;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Actor {
  pub id: Id<Actor>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Actor {
  pub fn new(id: Id<Actor>) -> Self {
    let now = Utc::now();
    Self {
      id,
      created_at: now,
      updated_at: now,
    }
  }
}
