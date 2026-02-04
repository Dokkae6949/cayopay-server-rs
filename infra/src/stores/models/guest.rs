use chrono::{DateTime, Utc};
use domain::{ActorId, Email, Guest};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct GuestRow {
  pub id: Uuid,
  pub actor_id: Uuid,
  pub email: Option<String>,
  pub verified: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct GuestCreation {
  pub actor_id: ActorId,
  pub email: Email,
  pub verified: bool,
}

#[derive(Clone)]
pub struct GuestUpdate {
  pub email: Option<Email>,
  pub verified: Option<bool>,
}

impl From<GuestRow> for Guest {
  fn from(value: GuestRow) -> Self {
    Self {
      id: value.id.into(),
      actor_id: value.actor_id.into(),
      email: value.email.map(Into::into),
      verified: value.verified,
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
