use chrono::{DateTime, Utc};
use domain::{Email, InviteId, InviteStatus, Role, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Request to send an invite
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct InviteRequest {
  #[validate(email(message = "Invalid email format"))]
  pub email: String,
  pub role: Role,
}

/// Request to accept an invite
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AcceptInviteRequest {
  #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
  pub password: String,
  #[validate(length(min = 1, message = "First name is required"))]
  pub first_name: String,
  #[validate(length(min = 1, message = "Last name is required"))]
  pub last_name: String,
}

/// Response with invite details (rich data, not just IDs)
#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
  pub id: InviteId,
  pub invitor_id: UserId,
  pub invitor_name: String,
  pub email: Email,
  pub token: String,
  pub role: Role,
  pub status: InviteStatus,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
