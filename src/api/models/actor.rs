use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
  api::models::{GuestResponse, UserResponse},
  domain::{actor::ActorWithDetails, Actor},
  types::Id,
};

#[derive(Serialize, ToSchema)]
pub struct ActorResponse {
  pub id: Id<Actor>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<UserResponse>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub guest: Option<GuestResponse>,
}

impl From<ActorWithDetails> for ActorResponse {
  fn from(details: ActorWithDetails) -> Self {
    let user = details.user.map(Into::into);
    let guest = details.guest.map(Into::into);

    Self {
      id: details.actor.id,
      created_at: details.actor.created_at,
      updated_at: details.actor.updated_at,
      user,
      guest,
    }
  }
}
