use sqlx::PgPool;

use crate::shared::error::AppResult;
use crate::shared::stores::UserStore;

use super::models::UserDetailResponse;

/// Service for user management operations
#[derive(Clone)]
pub struct UserManagementService {
  pool: PgPool,
}

impl UserManagementService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Get all users with full details
  pub async fn list_users(&self) -> AppResult<Vec<UserDetailResponse>> {
    let users = UserStore::list_all(&self.pool).await?;

    let responses = users
      .into_iter()
      .map(|user| UserDetailResponse {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        role: user.role,
        created_at: user.created_at,
        updated_at: user.updated_at,
      })
      .collect();

    Ok(responses)
  }
}
