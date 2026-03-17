use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{models::permission::PermissionId, Id, UserId};

pub type RoleId = Id<Role>;

/// A role is a named set of permissions that can be assigned to users.
/// Roles are stored in the database and can be created and modified at runtime.
/// Roles can optionally inherit all permissions from another role.
#[derive(Debug, Clone)]
pub struct Role {
  pub id: RoleId,
  pub name: String,
  pub description: Option<String>,
  pub inherited_from_role_id: Option<RoleId>,
  pub created_at: DateTime<Utc>,
}

pub type RolePermissionId = Id<RolePermission>;

/// Links a role to a permission with an optional scope.
///
/// - When `scope_kind` is `"global"` and `scope_id` is `None`, the permission
///   applies everywhere with no resource restriction.
/// - When `scope_kind` is e.g. `"shop"` and `scope_id` is `Some(uuid)`, the
///   permission applies only to that specific shop.
///
/// This enables checks like: "can user transfer funds **in shop:123**?"
#[derive(Debug, Clone)]
pub struct RolePermission {
  pub id: RolePermissionId,
  pub role_id: RoleId,
  pub permission_id: PermissionId,
  /// Resource kind this permission is scoped to: "global", "shop", "register", "event", etc.
  pub scope_kind: String,
  /// The specific resource UUID. `None` means the permission applies to all resources of that kind.
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Id;

  fn make_role(name: &str) -> Role {
    Role {
      id: Id::new(),
      name: name.to_string(),
      description: None,
      inherited_from_role_id: None,
      created_at: Utc::now(),
    }
  }

  #[test]
  fn test_role_created() {
    let role = make_role("owner");
    assert_eq!(role.name, "owner");
    assert!(role.description.is_none());
    assert!(role.inherited_from_role_id.is_none());
  }

  #[test]
  fn test_role_with_inheritance() {
    let parent = make_role("base");
    let child = Role {
      id: Id::new(),
      name: "child".to_string(),
      description: None,
      inherited_from_role_id: Some(parent.id),
      created_at: Utc::now(),
    };
    assert_eq!(child.inherited_from_role_id, Some(parent.id));
  }
}
