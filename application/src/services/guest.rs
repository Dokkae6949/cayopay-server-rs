use sqlx::PgPool;

use crate::{error::AppResult, services::AuthorizationService};
use domain::{models::permission::Permission, Guest, UserId};
use infra::stores::GuestStore;

#[derive(Clone)]
pub struct GuestService {
  pool: PgPool,
  authz_service: AuthorizationService,
}

impl GuestService {
  pub fn new(pool: PgPool, authz_service: AuthorizationService) -> Self {
    Self { pool, authz_service }
  }

  pub async fn get_all(&self, caller: UserId) -> AppResult<Vec<Guest>> {
    self.authz_service.require(caller, Permission::ReadGuest).await?;
    Ok(GuestStore::list_all(&self.pool).await?)
  }
}
