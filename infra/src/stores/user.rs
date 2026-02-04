use sqlx::{Executor, Postgres};

use crate::stores::models::user::{UserCreation, UserRow, UserUpdate};
use domain::{ActorId, Email, User, UserId};

pub struct UserStore;

impl UserStore {
  pub async fn create<'c, E>(executor: E, creation: &UserCreation) -> Result<User, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      INSERT INTO users (actor_id, email, password_hash, first_name, last_name, role)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      "#,
      creation.actor_id.into_inner(),
      creation.email.expose(),
      creation.password.expose(),
      creation.first_name,
      creation.last_name,
      creation.role.to_string(),
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id<'c, E>(
    executor: E,
    id: &UserId,
    update: &UserUpdate,
  ) -> Result<Option<User>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      UPDATE users
      SET email = COALESCE($2, email),
          password_hash = COALESCE($3, password_hash),
          first_name = COALESCE($4, first_name),
          last_name = COALESCE($5, last_name),
          role = COALESCE($6, role)
      WHERE id = $1
      RETURNING id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      "#,
      id.into_inner(),
      update.email.as_ref().map(|e| e.expose()),
      update.password.as_ref().map(|p| p.expose()),
      update.first_name.as_ref(),
      update.last_name.as_ref(),
      update.role.as_ref().map(ToString::to_string),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &UserId) -> Result<Option<User>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      FROM users
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_email<'c, E>(executor: E, email: &Email) -> Result<Option<User>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      FROM users
      WHERE email = $1
      "#,
      email.expose()
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_actor_id<'c, E>(
    executor: E,
    actor_id: &ActorId,
  ) -> Result<Option<User>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      FROM users
      WHERE actor_id = $1
      "#,
      actor_id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all<'c, E>(executor: E) -> Result<Vec<User>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      UserRow,
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      FROM users
      "#
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
