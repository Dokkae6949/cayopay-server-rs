use chrono::{DateTime, Utc};
use domain::{Email, GuestId};
use serde::Serialize;
use utoipa::ToSchema;

/// Rich guest response with all details
#[derive(Debug, Serialize, ToSchema)]
pub struct GuestDetailResponse {
  pub id: GuestId,
  pub email: Option<Email>,
  pub verified: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
