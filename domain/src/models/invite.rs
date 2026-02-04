use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{Email, Id, Role, UserId};

pub type InviteId = Id<Invite>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InviteStatus {
  #[default]
  Pending,
  Accepted,
  Declined,
  Revoked,
}

#[derive(Debug, Clone)]
pub struct Invite {
  pub id: InviteId,
  pub invitor: UserId,
  pub email: Email,
  pub token: String,
  pub role: Role,
  pub status: InviteStatus,
  pub expires_in: Duration,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

impl Invite {
  pub fn is_expired(&self) -> bool {
    Utc::now() > self.created_at + self.expires_in
  }
}

impl Display for InviteStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let status_str = match self {
      InviteStatus::Pending => "pending",
      InviteStatus::Accepted => "accepted",
      InviteStatus::Declined => "declined",
      InviteStatus::Revoked => "revoked",
    };
    write!(f, "{}", status_str)
  }
}

impl From<&str> for InviteStatus {
  fn from(value: &str) -> Self {
    match value {
      "pending" => InviteStatus::Pending,
      "accepted" => InviteStatus::Accepted,
      "declined" => InviteStatus::Declined,
      "revoked" => InviteStatus::Revoked,
      _ => InviteStatus::Pending,
    }
  }
}
