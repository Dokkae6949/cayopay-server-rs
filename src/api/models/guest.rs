use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
  domain::{Actor, Guest, Role},
  types::{Email, Id},
};

#[derive(Serialize, ToSchema)]
pub struct GuestResponse {
  pub id: Id<Guest>,
  pub actor_id: Id<Actor>,
}

impl From<Guest> for GuestResponse {
  fn from(guest: Guest) -> Self {
    Self {
      id: guest.id,
      actor_id: guest.actor_id,
    }
  }
}
