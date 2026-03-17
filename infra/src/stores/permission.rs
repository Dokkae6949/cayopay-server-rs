use domain::{Permission, PermissionId};
use sqlx::{Executor, Postgres};

use crate::stores::models::permission::{PermissionCreation, PermissionRow};

pub struct PermissionStore;

impl PermissionStore {
  pub async fn create<'c, E>(
    executor: E,
    creation: &PermissionCreation,
  ) -> Result<Permission, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, PermissionRow>(
      r#"
      INSERT INTO permissions (action, subject, description)
      VALUES ($1, $2, $3)
      RETURNING id, action, subject, description, created_at
      "#,
    )
    .bind(&creation.action)
    .bind(&creation.subject)
    .bind(creation.description.as_ref())
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn find_by_id<'c, E>(
    executor: E,
    id: &PermissionId,
  ) -> Result<Option<Permission>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, PermissionRow>(
      r#"
      SELECT id, action, subject, description, created_at
      FROM permissions
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_action_subject<'c, E>(
    executor: E,
    action: &str,
    subject: &str,
  ) -> Result<Option<Permission>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, PermissionRow>(
      r#"
      SELECT id, action, subject, description, created_at
      FROM permissions
      WHERE action = $1 AND subject = $2
      "#,
    )
    .bind(action)
    .bind(subject)
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all<'c, E>(executor: E) -> Result<Vec<Permission>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as::<_, PermissionRow>(
      r#"
      SELECT id, action, subject, description, created_at
      FROM permissions
      ORDER BY action, subject
      "#,
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }

  pub async fn delete_by_id<'c, E>(
    executor: E,
    id: &PermissionId,
  ) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query("DELETE FROM permissions WHERE id = $1")
      .bind(id.into_inner())
      .execute(executor)
      .await?;

    Ok(())
  }
}
