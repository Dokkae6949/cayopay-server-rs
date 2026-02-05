use sqlx::PgPool;

use crate::shared::error::AppResult;
use crate::shared::stores::GuestStore;

use super::models::GuestDetailResponse;

/// Service for guest management operations
#[derive(Clone)]
pub struct GuestManagementService {
  pool: PgPool,
}

impl GuestManagementService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Get all guests with full details
  pub async fn list_guests(&self) -> AppResult<Vec<GuestDetailResponse>> {
    let guests = GuestStore::list_all(&self.pool).await?;

    let responses = guests
      .into_iter()
      .map(|guest| GuestDetailResponse {
        id: guest.id,
        email: guest.email,
        verified: guest.verified,
        created_at: guest.created_at,
        updated_at: guest.updated_at,
      })
      .collect();

    Ok(responses)
  }
}
