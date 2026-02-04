use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use domain::{Actor, Email, Guest, Id};

#[derive(Serialize, ToSchema)]
pub struct GuestResponse {
  pub id: Id<Guest>,
  pub actor_id: Id<Actor>,
  pub email: Option<Email>,
  pub verified: bool,
  pub created_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at: Option<DateTime<Utc>>,
}

impl From<Guest> for GuestResponse {
  fn from(guest: Guest) -> Self {
    Self {
      id: guest.id,
      actor_id: guest.actor_id,
      email: guest.email,
      verified: guest.verified,
      created_at: guest.created_at,
      updated_at: guest.updated_at,
    }
  }
}
