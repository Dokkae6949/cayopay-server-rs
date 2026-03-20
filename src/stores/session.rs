use crate::models::{Session, UserId};
use crate::stores::models::session::{SessionCreation, SessionRow};
use sqlx::PgConnection;

pub struct SessionStore;

impl SessionStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &SessionCreation,
  ) -> Result<Session, sqlx::Error> {
    let row = sqlx::query_as::<_, SessionRow>(
      r#"
      INSERT INTO sessions (user_id, token, user_agent, ip_address, expires_at)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, user_id, token, user_agent, ip_address, expires_at, created_at, updated_at
      "#,
    )
    .bind(creation.user_id.into_inner())
    .bind(&creation.token)
    .bind(&creation.user_agent)
    .bind(&creation.ip_address)
    .bind(chrono::Utc::now() + creation.expires_in)
    .fetch_one(conn)
    .await?;

    Ok(row.into())
  }

  pub async fn delete_by_token(conn: &mut PgConnection, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
      .bind(token)
      .execute(conn)
      .await?;

    Ok(())
  }

  pub async fn find_by_token(
    conn: &mut PgConnection,
    token: &str,
  ) -> Result<Option<Session>, sqlx::Error> {
    let row = sqlx::query_as::<_, SessionRow>(
      r#"
      SELECT id, user_id, token, user_agent, ip_address, expires_at, created_at, updated_at
      FROM sessions
      WHERE token = $1
      "#,
    )
    .bind(token)
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_by_user_id(
    conn: &mut PgConnection,
    user_id: &UserId,
  ) -> Result<Vec<Session>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SessionRow>(
      r#"
      SELECT id, user_id, token, user_agent, ip_address, expires_at, created_at, updated_at
      FROM sessions
      WHERE user_id = $1
      "#,
    )
    .bind(user_id.into_inner())
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
