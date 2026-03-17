use sqlx::PgPool;

use crate::{error::AppResult, services::AuthorizationService};
use domain::{models::permission::Permission, User, UserId};
use infra::stores::UserStore;

pub async fn find_by_id(pool: &PgPool, id: UserId) -> AppResult<Option<User>> {
  Ok(UserStore::find_by_id(pool, &id).await?)
}

pub async fn list_all(
  pool: &PgPool,
  authz: &AuthorizationService,
  caller: UserId,
) -> AppResult<Vec<User>> {
  authz.require(caller, Permission::ReadUser).await?;
  Ok(UserStore::list_all(pool).await?)
}
