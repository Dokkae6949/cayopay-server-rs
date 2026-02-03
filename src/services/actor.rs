use sqlx::PgPool;

use crate::{
  domain::{actor::ActorWithDetails, Actor, User},
  error::AppResult,
  stores::{ActorStore, UserStore},
  types::Id,
};

#[derive(Clone)]
pub struct ActorService {
  pool: PgPool,
}

impl ActorService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn get_user_by_id(&self, user_id: &Id<User>) -> AppResult<Option<User>> {
    UserStore::find_by_id(&self.pool, user_id).await
  }

  pub async fn list_actors(&self) -> AppResult<Vec<ActorWithDetails>> {
    ActorStore::list_all_detailed(&self.pool).await
  }

  pub async fn get_actor_by_id(&self, actor_id: &Id<Actor>) -> AppResult<Option<ActorWithDetails>> {
    ActorStore::find_by_id_detailed(&self.pool, actor_id).await
  }

  pub async fn remove_by_id(&self, id: Id<Actor>) -> AppResult<()> {
    ActorStore::remove_by_id(&self.pool, &id).await?;

    Ok(())
  }
}
