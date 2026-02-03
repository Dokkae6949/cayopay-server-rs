pub mod actor;
pub mod auth;
pub mod guest;
pub mod health;
pub mod invite;
pub mod user;

pub use actor::*;
pub use auth::*;
pub use guest::*;
pub use health::*;
pub use invite::*;
pub use user::*;

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
  domain::{Guest, Role, User},
  types::{Email, Id},
};

#[derive(Serialize, ToSchema)]
pub struct GuestResponse {
  pub id: Id<Guest>,
  pub actor_id: Id<crate::domain::Actor>,
}

impl From<Guest> for GuestResponse {
  fn from(guest: Guest) -> Self {
    Self {
      id: guest.id,
      actor_id: guest.actor_id,
    }
  }
}
