use sqlx::PgPool;

use crate::{
  domain::User,
  error::AppResult,
  stores::UserStore,
  types::Id,
};

#[derive(Clone)]
pub struct UserService {
  pool: PgPool,
}

impl UserService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn get_by_id(&self, user_id: &Id<User>) -> AppResult<Option<User>> {
    UserStore::find_by_id(&self.pool, user_id).await
  }

  pub async fn list_all(&self) -> AppResult<Vec<User>> {
    UserStore::list_all(&self.pool).await
  }

  pub async fn remove_by_id(&self, id: Id<User>) -> AppResult<()> {
    UserStore::remove_by_id(&self.pool, &id).await
  }
}
