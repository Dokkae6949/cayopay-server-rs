use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::UserId;
use crate::types::Id;

pub type RoleId = Id<Role>;

/// A role is a named set of permissions that can be assigned to users.
/// Roles are stored in the database and can be created and modified at runtime.
#[derive(Debug, Clone)]
pub struct Role {
  pub id: RoleId,
  pub name: String,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
}

pub type RolePermissionId = Id<RolePermission>;

/// Links a role to a permission string with an optional resource scope.
///
/// - When `scope_kind` is `None` the permission applies globally with no resource restriction.
/// - When `scope_kind` is `Some("shop")` and `scope_id` is `Some(uuid)`, the permission
///   applies only to that specific shop.
/// - When `scope_kind` is `Some("shop")` and `scope_id` is `None`, the permission applies
///   to all shops.
#[derive(Debug, Clone)]
pub struct RolePermission {
  pub id: RolePermissionId,
  pub role_id: RoleId,
  /// The permission string code (e.g. `"settings.configure"`).
  pub permission: String,
  /// Resource kind this permission is scoped to: `"shop"`, etc.
  /// `None` means the permission applies globally.
  pub scope_kind: Option<String>,
  /// The specific resource UUID. `None` means all resources of `scope_kind`.
  pub scope_id: Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

pub type UserRoleId = Id<UserRole>;

/// Links a user to a role, granting all permissions attached to that role.
#[derive(Debug, Clone)]
pub struct UserRole {
  pub id: UserRoleId,
  pub user_id: UserId,
  pub role_id: RoleId,
  pub created_at: DateTime<Utc>,
}
