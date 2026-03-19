use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::{permission::Permission, Role, RoleId, RolePermission, UserRole, UserId};

#[derive(Clone, FromRow)]
pub struct RoleRow {
  pub id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RoleCreation {
  pub name: String,
  pub description: Option<String>,
}

/// DB row for `role_permissions`.
/// The `permission` column stores the canonical string code of a [`Permission`] variant.
#[derive(Clone, FromRow)]
pub struct RolePermissionRow {
  pub id: Uuid,
  pub role_id: Uuid,
  pub permission: String,
  pub scope_kind: Option<String>,
  pub scope_id: Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RolePermissionCreation {
  pub role_id: RoleId,
  pub permission: Permission,
  pub scope_kind: Option<String>,
  pub scope_id: Option<Uuid>,
}

#[derive(Clone, FromRow)]
pub struct UserRoleRow {
  pub id: Uuid,
  pub user_id: Uuid,
  pub role_id: Uuid,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct UserRoleCreation {
  pub user_id: UserId,
  pub role_id: RoleId,
}

impl From<RoleRow> for Role {
  fn from(value: RoleRow) -> Self {
    Self {
      id: value.id.into(),
      name: value.name,
      description: value.description,
      created_at: value.created_at,
    }
  }
}

impl TryFrom<RolePermissionRow> for RolePermission {
  type Error = String;

  fn try_from(value: RolePermissionRow) -> Result<Self, Self::Error> {
    let permission = Permission::from_code(&value.permission)
      .ok_or_else(|| format!("unknown permission code '{}' in database", value.permission))?;
    Ok(Self {
      id: value.id.into(),
      role_id: value.role_id.into(),
      permission,
      scope_kind: value.scope_kind,
      scope_id: value.scope_id,
      created_at: value.created_at,
    })
  }
}

impl From<UserRoleRow> for UserRole {
  fn from(value: UserRoleRow) -> Self {
    Self {
      id: value.id.into(),
      user_id: value.user_id.into(),
      role_id: value.role_id.into(),
      created_at: value.created_at,
    }
  }
}
