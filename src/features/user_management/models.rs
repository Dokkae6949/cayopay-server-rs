use chrono::{DateTime, Utc};
use domain::{Email, Role, UserId};
use serde::Serialize;
use utoipa::ToSchema;

/// Rich user response with all details
#[derive(Debug, Serialize, ToSchema)]
pub struct UserDetailResponse {
  pub id: UserId,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
