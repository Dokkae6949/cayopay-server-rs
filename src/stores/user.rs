use crate::error::AppResult;
use crate::types::Email;
use crate::{domain::User, types::Id};
use sqlx::{Executor, PgConnection, Postgres};

pub struct UserStore;

impl UserStore {
  pub async fn save<'c, E>(executor: E, user: &User) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      INSERT INTO users (id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      "#,
      user.id.into_inner(),
      user.actor_id.into_inner(),
      user.email.expose(),
      user.password_hash.expose(),
      user.first_name,
      user.last_name,
      user.role.to_string(),
      user.created_at,
      user.updated_at,
    )
    .execute(executor)
    .await
    .map_err(|e| match &e {
      sqlx::Error::Database(db_err) => match db_err.kind() {
        sqlx::error::ErrorKind::UniqueViolation => crate::error::AppError::UserAlreadyExists,
        _ => crate::error::AppError::Database(e),
      },
      _ => crate::error::AppError::Database(e),
    })?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &Id<User>) -> AppResult<Option<User>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let user = sqlx::query_as!(
      User,
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      FROM users
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(user)
  }

  pub async fn find_by_email<'c, E>(executor: E, email: &Email) -> AppResult<Option<User>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let user = sqlx::query_as!(
      User,
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at
      FROM users
      WHERE email = $1
      "#,
      email.expose()
    )
    .fetch_optional(executor)
    .await?;

    Ok(user)
  }
}
