use crate::models::{ActorId, User, UserId};
use crate::stores::models::user::{UserCreation, UserRow, UserUpdate};
use crate::types::Email;
use sqlx::PgConnection;

pub struct UserStore;

impl UserStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &UserCreation,
  ) -> Result<User, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
      r#"
      INSERT INTO users (actor_id, email, password_hash, first_name, last_name)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, actor_id, email, password_hash, first_name, last_name, created_at, updated_at
      "#,
    )
    .bind(creation.actor_id.into_inner())
    .bind(creation.email.expose())
    .bind(creation.password.expose())
    .bind(&creation.first_name)
    .bind(&creation.last_name)
    .fetch_one(&mut *conn)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id(
    conn: &mut PgConnection,
    id: &UserId,
    update: &UserUpdate,
  ) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
      r#"
      UPDATE users
      SET email = COALESCE($2, email),
          password_hash = COALESCE($3, password_hash),
          first_name = COALESCE($4, first_name),
          last_name = COALESCE($5, last_name)
      WHERE id = $1
      RETURNING id, actor_id, email, password_hash, first_name, last_name, created_at, updated_at
      "#,
    )
    .bind(id.into_inner())
    .bind(update.email.as_ref().map(|e| e.expose()))
    .bind(update.password.as_ref().map(|p| p.expose()))
    .bind(update.first_name.as_ref())
    .bind(update.last_name.as_ref())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &UserId,
  ) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, created_at, updated_at
      FROM users
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_email(
    conn: &mut PgConnection,
    email: &Email,
  ) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, created_at, updated_at
      FROM users
      WHERE email = $1
      "#,
    )
    .bind(email.expose())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_actor_id(
    conn: &mut PgConnection,
    actor_id: &ActorId,
  ) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, created_at, updated_at
      FROM users
      WHERE actor_id = $1
      "#,
    )
    .bind(actor_id.into_inner())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all(conn: &mut PgConnection) -> Result<Vec<User>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserRow>(
      r#"
      SELECT id, actor_id, email, password_hash, first_name, last_name, created_at, updated_at
      FROM users
      "#,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
