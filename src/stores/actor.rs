use sqlx::{Executor, Postgres};

use crate::domain::actor::{Actor, ActorWithDetails};
use crate::error::AppResult;
use crate::types::Id;

pub struct ActorStore;

impl ActorStore {
  pub async fn save<'c, E>(executor: E, actor: &Actor) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
            INSERT INTO actors (id, created_at, updated_at)
            VALUES ($1, $2, $3)
            "#,
      actor.id.into_inner(),
      actor.created_at,
      actor.updated_at
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &Id<Actor>) -> AppResult<Option<Actor>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let actor = sqlx::query_as!(
      Actor,
      r#"
        SELECT id, created_at, updated_at
        FROM actors
        WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(actor)
  }

  pub async fn remove_by_id<'c, E>(executor: E, id: &Id<Actor>) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM actors
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id_detailed<'c, E>(
    executor: E,
    id: &Id<Actor>,
  ) -> AppResult<Option<ActorWithDetails>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query!(
      r#"
      SELECT
        a.id as actor_id,
        a.created_at as actor_created_at,
        a.updated_at as actor_updated_at,
        u.id as "user_id?",
        u.actor_id as "user_actor_id?",
        u.email as "user_email?",
        u.password_hash as "user_password_hash?",
        u.first_name as "user_first_name?",
        u.last_name as "user_last_name?",
        u.role as "user_role?",
        u.created_at as "user_created_at?",
        u.updated_at as "user_updated_at?",
        g.id as "guest_id?",
        g.actor_id as "guest_actor_id?"
      FROM actors a
      LEFT JOIN users u ON a.id = u.actor_id
      LEFT JOIN guests g ON a.id = g.actor_id
      WHERE a.id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    if let Some(row) = row {
      let user = if let (
        Some(id),
        Some(actor_id),
        Some(email),
        Some(password_hash),
        Some(first_name),
        Some(last_name),
        Some(role),
        Some(created_at),
        Some(updated_at),
      ) = (
        row.user_id,
        row.user_actor_id,
        row.user_email,
        row.user_password_hash,
        row.user_first_name,
        row.user_last_name,
        row.user_role,
        row.user_created_at,
        row.user_updated_at,
      ) {
        Some(crate::domain::User {
          id: Id::from(id),
          actor_id: Id::from(actor_id),
          email: crate::types::Email::from(email),
          password_hash: crate::types::HashedPassword::from(password_hash),
          first_name,
          last_name,
          role: crate::domain::Role::from(role),
          created_at,
          updated_at,
        })
      } else {
        None
      };

      let guest = if let (Some(id), Some(actor_id)) = (row.guest_id, row.guest_actor_id) {
        Some(crate::domain::Guest {
          id: Id::from(id),
          actor_id: Id::from(actor_id),
        })
      } else {
        None
      };
      Ok(Some(ActorWithDetails {
        actor: Actor {
          id: Id::from(row.actor_id),
          created_at: row.actor_created_at,
          updated_at: row.actor_updated_at,
        },
        user,
        guest,
      }))
    } else {
      Ok(None)
    }
  }

  pub async fn list_all_detailed<'c, E>(executor: E) -> AppResult<Vec<ActorWithDetails>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query!(
      r#"
      SELECT
        a.id as actor_id,
        a.created_at as actor_created_at,
        a.updated_at as actor_updated_at,
        u.id as "user_id?",
        u.actor_id as "user_actor_id?",
        u.email as "user_email?",
        u.password_hash as "user_password_hash?",
        u.first_name as "user_first_name?",
        u.last_name as "user_last_name?",
        u.role as "user_role?",
        u.created_at as "user_created_at?",
        u.updated_at as "user_updated_at?",
        g.id as "guest_id?",
        g.actor_id as "guest_actor_id?"
      FROM actors a
      LEFT JOIN users u ON a.id = u.actor_id
      LEFT JOIN guests g ON a.id = g.actor_id
      "#
    )
    .fetch_all(executor)
    .await?;

    let actors = rows
      .into_iter()
      .map(|row| {
        let user = if let (
          Some(id),
          Some(actor_id),
          Some(email),
          Some(password_hash),
          Some(first_name),
          Some(last_name),
          Some(role),
          Some(created_at),
          Some(updated_at),
        ) = (
          row.user_id,
          row.user_actor_id,
          row.user_email,
          row.user_password_hash,
          row.user_first_name,
          row.user_last_name,
          row.user_role,
          row.user_created_at,
          row.user_updated_at,
        ) {
          Some(crate::domain::User {
            id: Id::from(id),
            actor_id: Id::from(actor_id),
            email: crate::types::Email::from(email),
            password_hash: crate::types::HashedPassword::from(password_hash),
            first_name,
            last_name,
            role: crate::domain::Role::from(role),
            created_at,
            updated_at,
          })
        } else {
          None
        };

        let guest = if let (Some(id), Some(actor_id)) = (row.guest_id, row.guest_actor_id) {
          Some(crate::domain::Guest {
            id: Id::from(id),
            actor_id: Id::from(actor_id),
          })
        } else {
          None
        };

        ActorWithDetails {
          actor: Actor {
            id: Id::from(row.actor_id),
            created_at: row.actor_created_at,
            updated_at: row.actor_updated_at,
          },
          user,
          guest,
        }
      })
      .collect();

    Ok(actors)
  }
}
