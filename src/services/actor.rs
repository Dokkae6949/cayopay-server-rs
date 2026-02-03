use std::collections::HashMap;

use sqlx::PgPool;

use crate::{
  domain::{actor::ActorWithDetails, Actor},
  error::AppResult,
  stores::{ActorStore, GuestStore, UserStore},
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

  pub async fn list_actors(&self) -> AppResult<Vec<ActorWithDetails>> {
    let actors = ActorStore::list_all(&self.pool).await?;
    let users = UserStore::list_all(&self.pool).await?;
    let guests = GuestStore::list_all(&self.pool).await?;

    let mut users_map: HashMap<_, _> = users.into_iter().map(|u| (u.actor_id, u)).collect();
    let mut guests_map: HashMap<_, _> = guests.into_iter().map(|g| (g.actor_id, g)).collect();

    let actors_with_details = actors
      .into_iter()
      .map(|actor| {
        let user = users_map.remove(&actor.id);
        let guest = guests_map.remove(&actor.id);

        ActorWithDetails { actor, user, guest }
      })
      .collect();

    Ok(actors_with_details)
  }

  pub async fn get_actor_by_id(&self, actor_id: &Id<Actor>) -> AppResult<Option<ActorWithDetails>> {
    let actor = ActorStore::find_by_id(&self.pool, actor_id).await?;

    match actor {
      Some(actor) => {
        let user = UserStore::find_by_actor_id(&self.pool, actor_id).await?;
        let guest = GuestStore::find_by_actor_id(&self.pool, actor_id).await?;

        Ok(Some(ActorWithDetails { actor, user, guest }))
      }
      None => Ok(None),
    }
  }

  pub async fn remove_by_id(&self, id: Id<Actor>) -> AppResult<()> {
    ActorStore::remove_by_id(&self.pool, &id).await
  }
}

