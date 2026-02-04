use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use domain::{Id, Invite, InviteStatus, Role, User};

#[derive(Deserialize, Validate, ToSchema)]
pub struct InviteRequest {
  #[validate(email)]
  #[schema(example = "friend@example.com")]
  pub email: String,

  pub role: Role,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct AcceptInviteRequest {
  #[validate(length(min = 1, max = 127))]
  #[schema(example = "John")]
  pub first_name: String,
  #[validate(length(min = 1, max = 127))]
  #[schema(example = "Doe")]
  pub last_name: String,
  #[validate(length(min = 8, max = 127))]
  #[schema(example = "password123")]
  pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct InviteResponse {
  pub id: Id<Invite>,
  pub invitor: Id<User>,
  pub email: String,
  pub role: Role,
  pub status: InviteStatus,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at: Option<DateTime<Utc>>,
}

impl From<Invite> for InviteResponse {
  fn from(invite: Invite) -> Self {
    Self {
      id: invite.id,
      invitor: invite.invitor,
      email: invite.email.expose().to_string(),
      role: invite.role,
      status: invite.status,
      expires_at: invite.created_at + invite.expires_in,
      created_at: invite.created_at,
      updated_at: invite.updated_at,
    }
  }
}
