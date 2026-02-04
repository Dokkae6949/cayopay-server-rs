use sqlx::PgPool;

use crate::error::AppResult;
use domain::{User, UserId};
use infra::stores::UserStore;

#[derive(Clone)]
pub struct UserService {
  pool: PgPool,
}

impl UserService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn get_by_id(&self, id: UserId) -> AppResult<Option<User>> {
    Ok(UserStore::find_by_id(&self.pool, &id).await?)
  }

  pub async fn get_all(&self) -> AppResult<Vec<User>> {
    Ok(UserStore::list_all(&self.pool).await?)
  }
}
