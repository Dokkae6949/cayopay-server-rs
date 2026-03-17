use sqlx::PgPool;

use crate::{error::AppResult, services::AuthorizationService};
use domain::{models::permission::Permission, User, UserId};
use infra::stores::UserStore;

#[derive(Clone)]
pub struct UserService {
  pool: PgPool,
  authz_service: AuthorizationService,
}

impl UserService {
  pub fn new(pool: PgPool, authz_service: AuthorizationService) -> Self {
    Self { pool, authz_service }
  }

  pub async fn get_by_id(&self, id: UserId) -> AppResult<Option<User>> {
    Ok(UserStore::find_by_id(&self.pool, &id).await?)
  }

  pub async fn get_all(&self, caller: UserId) -> AppResult<Vec<User>> {
    self.authz_service.require(caller, Permission::ReadUser).await?;
    Ok(UserStore::list_all(&self.pool).await?)
  }
}
