use sqlx::PgPool;

use crate::{
  domain::Guest,
  error::AppResult,
  stores::GuestStore,
  types::Id,
};

#[derive(Clone)]
pub struct GuestService {
  pool: PgPool,
}

impl GuestService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn list_all(&self) -> AppResult<Vec<Guest>> {
    GuestStore::list_all(&self.pool).await
  }

  pub async fn remove_by_id(&self, id: Id<Guest>) -> AppResult<()> {
    GuestStore::remove_by_id(&self.pool, &id).await
  }
}
