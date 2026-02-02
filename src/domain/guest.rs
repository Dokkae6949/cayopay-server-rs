use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::Actor;
use crate::types::Id;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Guest {
  pub id: Id<Guest>,
  pub actor_id: Id<Actor>,
}

impl Guest {
  pub fn new(actor_id: Id<Actor>) -> Self {
    Self {
      id: Id::new(),
      actor_id,
    }
  }
}
