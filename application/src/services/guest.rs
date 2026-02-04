use sqlx::PgPool;

use crate::error::AppResult;
use domain::Guest;
use infra::stores::GuestStore;

#[derive(Clone)]
pub struct GuestService {
  pool: PgPool,
}

impl GuestService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn get_all(&self) -> AppResult<Vec<Guest>> {
    Ok(GuestStore::get_all(&self.pool).await?)
  }
}
