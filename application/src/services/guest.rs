use sqlx::PgPool;

use crate::{error::AppResult, services::AuthorizationService};
use domain::{models::permission::Permission, Guest, UserId};
use infra::stores::GuestStore;

pub async fn list_all(
  pool: &PgPool,
  authz: &AuthorizationService,
  caller: UserId,
) -> AppResult<Vec<Guest>> {
  authz.require(caller, Permission::ReadGuest).await?;
  Ok(GuestStore::list_all(pool).await?)
}
