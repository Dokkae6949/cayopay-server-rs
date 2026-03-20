use crate::models::{Role, RoleId, RolePermission, RolePermissionId, UserId, UserRole, UserRoleId};
use crate::stores::models::role::{
  RoleCreation, RolePermissionCreation, RolePermissionRow, RoleRow, UserRoleCreation, UserRoleRow,
};
use sqlx::PgConnection;

pub struct RoleStore;

impl RoleStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &RoleCreation,
  ) -> Result<Role, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
      r#"
      INSERT INTO roles (name, description)
      VALUES ($1, $2)
      RETURNING id, name, description, created_at
      "#,
    )
    .bind(&creation.name)
    .bind(creation.description.as_ref())
    .fetch_one(&mut *conn)
    .await?;

    Ok(row.into())
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &RoleId,
  ) -> Result<Option<Role>, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
      r#"
      SELECT id, name, description, created_at
      FROM roles
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_name(
    conn: &mut PgConnection,
    name: &str,
  ) -> Result<Option<Role>, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
      r#"
      SELECT id, name, description, created_at
      FROM roles
      WHERE name = $1
      "#,
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all(conn: &mut PgConnection) -> Result<Vec<Role>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RoleRow>(
      r#"
      SELECT id, name, description, created_at
      FROM roles
      ORDER BY name
      "#,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }

  pub async fn delete_by_id(conn: &mut PgConnection, id: &RoleId) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM roles WHERE id = $1")
      .bind(id.into_inner())
      .execute(&mut *conn)
      .await?;

    Ok(())
  }
}

pub struct RolePermissionStore;

impl RolePermissionStore {
  pub async fn add(
    conn: &mut PgConnection,
    creation: &RolePermissionCreation,
  ) -> Result<RolePermission, sqlx::Error> {
    let row = sqlx::query_as::<_, RolePermissionRow>(
      r#"
      INSERT INTO role_permissions (role_id, permission, scope_kind, scope_id)
      VALUES ($1, $2, $3, $4)
      RETURNING id, role_id, permission, scope_kind, scope_id, created_at
      "#,
    )
    .bind(creation.role_id.into_inner())
    .bind(&creation.permission)
    .bind(creation.scope_kind.as_deref())
    .bind(creation.scope_id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(row.into())
  }

  pub async fn remove(conn: &mut PgConnection, id: &RolePermissionId) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM role_permissions WHERE id = $1")
      .bind(id.into_inner())
      .execute(&mut *conn)
      .await?;

    Ok(())
  }

  /// Lists all permissions directly attached to a role.
  pub async fn list_for_role(
    conn: &mut PgConnection,
    role_id: &RoleId,
  ) -> Result<Vec<RolePermission>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RolePermissionRow>(
      r#"
      SELECT id, role_id, permission, scope_kind, scope_id, created_at
      FROM role_permissions
      WHERE role_id = $1
      "#,
    )
    .bind(role_id.into_inner())
    .fetch_all(&mut *conn)
    .await?;

    rows.into_iter().map(|r| Ok(r.into())).collect()
  }
}

pub struct UserRoleStore;

impl UserRoleStore {
  pub async fn assign(
    conn: &mut PgConnection,
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
    .fetch_optional(&mut *conn)
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
    .fetch_one(&mut *conn)
    .await?;

    Ok(row.into())
  }

  pub async fn revoke(conn: &mut PgConnection, id: &UserRoleId) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_roles WHERE id = $1")
      .bind(id.into_inner())
      .execute(&mut *conn)
      .await?;

    Ok(())
  }

  pub async fn revoke_by_user_and_role(
    conn: &mut PgConnection,
    user_id: &UserId,
    role_id: &RoleId,
  ) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
      .bind(user_id.into_inner())
      .bind(role_id.into_inner())
      .execute(&mut *conn)
      .await?;

    Ok(())
  }

  pub async fn list_for_user(
    conn: &mut PgConnection,
    user_id: &UserId,
  ) -> Result<Vec<UserRole>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserRoleRow>(
      r#"
      SELECT id, user_id, role_id, created_at
      FROM user_roles
      WHERE user_id = $1
      "#,
    )
    .bind(user_id.into_inner())
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
