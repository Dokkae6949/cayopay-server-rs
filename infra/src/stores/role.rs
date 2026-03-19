use domain::{Role, RoleId, RolePermission, RolePermissionId, UserRole, UserRoleId, UserId};
use sqlx::{Executor, Postgres};

use crate::stores::models::role::{
  RoleCreation, RolePermissionCreation, RolePermissionRow, RoleRow, UserRoleCreation, UserRoleRow,
};

pub struct RoleStore;

impl RoleStore {
  pub async fn create<'c, E>(executor: E, creation: &RoleCreation) -> Result<Role, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, RoleRow>(
      r#"
      INSERT INTO roles (name, description)
      VALUES ($1, $2)
      RETURNING id, name, description, created_at
      "#,
    )
    .bind(&creation.name)
    .bind(creation.description.as_ref())
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &RoleId) -> Result<Option<Role>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, RoleRow>(
      r#"
      SELECT id, name, description, created_at
      FROM roles
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_name<'c, E>(
    executor: E,
    name: &str,
  ) -> Result<Option<Role>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, RoleRow>(
      r#"
      SELECT id, name, description, created_at
      FROM roles
      WHERE name = $1
      "#,
    )
    .bind(name)
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all<'c, E>(executor: E) -> Result<Vec<Role>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as::<_, RoleRow>(
      r#"
      SELECT id, name, description, created_at
      FROM roles
      ORDER BY name
      "#,
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }

  pub async fn delete_by_id<'c, E>(executor: E, id: &RoleId) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query("DELETE FROM roles WHERE id = $1")
      .bind(id.into_inner())
      .execute(executor)
      .await?;

    Ok(())
  }
}

pub struct RolePermissionStore;

impl RolePermissionStore {
  pub async fn add<'c, E>(
    executor: E,
    creation: &RolePermissionCreation,
  ) -> Result<RolePermission, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as::<_, RolePermissionRow>(
      r#"
      INSERT INTO role_permissions (role_id, permission, scope_kind, scope_id)
      VALUES ($1, $2, $3, $4)
      RETURNING id, role_id, permission, scope_kind, scope_id, created_at
      "#,
    )
    .bind(creation.role_id.into_inner())
    .bind(creation.permission.as_str())
    .bind(creation.scope_kind.as_deref())
    .bind(creation.scope_id)
    .fetch_one(executor)
    .await?;

    row.try_into().map_err(|e: String| sqlx::Error::Decode(e.into()))
  }

  pub async fn remove<'c, E>(
    executor: E,
    id: &RolePermissionId,
  ) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query("DELETE FROM role_permissions WHERE id = $1")
      .bind(id.into_inner())
      .execute(executor)
      .await?;

    Ok(())
  }

  /// Lists all permissions directly attached to a role.
  pub async fn list_for_role<'c, E>(
    executor: E,
    role_id: &RoleId,
  ) -> Result<Vec<RolePermission>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as::<_, RolePermissionRow>(
      r#"
      SELECT id, role_id, permission, scope_kind, scope_id, created_at
      FROM role_permissions
      WHERE role_id = $1
      "#,
    )
    .bind(role_id.into_inner())
    .fetch_all(executor)
    .await?;

    rows
      .into_iter()
      .map(|r| r.try_into().map_err(|e: String| sqlx::Error::Decode(e.into())))
      .collect()
  }
}

pub struct UserRoleStore;

impl UserRoleStore {
  pub async fn assign(
    pool: &sqlx::PgPool,
    creation: &UserRoleCreation,
  ) -> Result<UserRole, sqlx::Error> {
    // Try to insert; on conflict (assignment already exists) fetch the existing row.
    let inserted = sqlx::query_as::<_, UserRoleRow>(
      r#"
      INSERT INTO user_roles (user_id, role_id)
      VALUES ($1, $2)
      ON CONFLICT (user_id, role_id) DO NOTHING
      RETURNING id, user_id, role_id, created_at
      "#,
    )
    .bind(creation.user_id.into_inner())
    .bind(creation.role_id.into_inner())
    .fetch_optional(pool)
    .await?;

    if let Some(row) = inserted {
      return Ok(row.into());
    }

    // Conflict: the assignment already exists; fetch and return it.
    let row = sqlx::query_as::<_, UserRoleRow>(
      r#"
      SELECT id, user_id, role_id, created_at
      FROM user_roles
      WHERE user_id = $1 AND role_id = $2
      "#,
    )
    .bind(creation.user_id.into_inner())
    .bind(creation.role_id.into_inner())
    .fetch_one(pool)
    .await?;

    Ok(row.into())
  }

  pub async fn revoke<'c, E>(
    executor: E,
    id: &UserRoleId,
  ) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query("DELETE FROM user_roles WHERE id = $1")
      .bind(id.into_inner())
      .execute(executor)
      .await?;

    Ok(())
  }

  pub async fn revoke_by_user_and_role<'c, E>(
    executor: E,
    user_id: &UserId,
    role_id: &RoleId,
  ) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
      .bind(user_id.into_inner())
      .bind(role_id.into_inner())
      .execute(executor)
      .await?;

    Ok(())
  }

  pub async fn list_for_user<'c, E>(
    executor: E,
    user_id: &UserId,
  ) -> Result<Vec<UserRole>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as::<_, UserRoleRow>(
      r#"
      SELECT id, user_id, role_id, created_at
      FROM user_roles
      WHERE user_id = $1
      "#,
    )
    .bind(user_id.into_inner())
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
