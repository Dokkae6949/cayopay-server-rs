use chrono::{DateTime, Duration, Utc};
use domain::{invite::InviteStatus, Email, Invite, Role, UserId};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct InviteRow {
  pub id: Uuid,
  pub invitor_user_id: Uuid,
  pub email: String,
  pub token: String,
  pub role: String,
  pub status: String,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct InviteCreation {
  pub invitor: UserId,
  pub email: Email,
  pub token: String,
  pub role: Role,
  pub expires_in: Duration,
}

#[derive(Clone)]
pub struct InviteUpdate {
  pub status: Option<InviteStatus>,
}

impl From<InviteRow> for Invite {
  fn from(value: InviteRow) -> Self {
    Self {
      id: value.id.into(),
      invitor: value.invitor_user_id.into(),
      email: value.email.into(),
      token: value.token,
      role: value.role.into(),
      status: value.status.as_str().into(),
      expires_in: value.expires_at - value.created_at,
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
