use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  ConfigureSettings,

  InviteUsers,
  ViewAllActors,
}

#[derive(
  Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema,
)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
  #[default]
  Undefined,

  Owner,
  Admin,
}

impl Display for Role {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      Role::Owner => "owner",
      Role::Admin => "admin",
      Role::Undefined => "undefined",
    };
    write!(f, "{}", s)
  }
}

impl From<String> for Role {
  fn from(s: String) -> Self {
    match s.as_str() {
      "owner" => Role::Owner,
      "admin" => Role::Admin,
      _ => Role::Undefined,
    }
  }
}

impl Role {
  pub fn permissions(&self) -> Vec<Permission> {
    match self {
      Role::Owner => vec![
        Permission::ConfigureSettings,
        Permission::InviteUsers,
        Permission::ViewAllActors,
      ],
      Role::Admin => vec![Permission::InviteUsers, Permission::ViewAllActors],
      Role::Undefined => vec![],
    }
  }

  pub fn has_permission(&self, perm: Permission) -> bool {
    self.permissions().contains(&perm)
  }

  pub fn can_assign_role(&self, target_role: Role) -> bool {
    match self {
      Role::Owner => matches!(target_role, Role::Owner | Role::Admin),
      Role::Admin => matches!(target_role, Role::Admin),
      Role::Undefined => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_role_permissions() {
    let owner_perms = Role::Owner.permissions();
    assert!(owner_perms.contains(&Permission::ConfigureSettings));
    assert!(owner_perms.contains(&Permission::InviteUsers));

    let admin_perms = Role::Admin.permissions();
    assert!(!admin_perms.contains(&Permission::ConfigureSettings));
    assert!(admin_perms.contains(&Permission::InviteUsers));

    let undefined_perms = Role::Undefined.permissions();
    assert!(undefined_perms.is_empty());
  }

  #[test]
  fn test_has_permission() {
    assert!(Role::Owner.has_permission(Permission::ConfigureSettings));
    assert!(Role::Owner.has_permission(Permission::InviteUsers));

    assert!(!Role::Admin.has_permission(Permission::ConfigureSettings));
    assert!(Role::Admin.has_permission(Permission::InviteUsers));

    assert!(!Role::Undefined.has_permission(Permission::ConfigureSettings));
    assert!(!Role::Undefined.has_permission(Permission::InviteUsers));
  }

  #[test]
  fn test_can_assign_role() {
    // Owner can assign Owner and Admin
    assert!(Role::Owner.can_assign_role(Role::Owner));
    assert!(Role::Owner.can_assign_role(Role::Admin));
    assert!(!Role::Owner.can_assign_role(Role::Undefined));

    // Admin can assign Admin only
    assert!(!Role::Admin.can_assign_role(Role::Owner));
    assert!(Role::Admin.can_assign_role(Role::Admin));
    assert!(!Role::Admin.can_assign_role(Role::Undefined));

    // Undefined can assign nothing
    assert!(!Role::Undefined.can_assign_role(Role::Owner));
    assert!(!Role::Undefined.can_assign_role(Role::Admin));
    assert!(!Role::Undefined.can_assign_role(Role::Undefined));
  }
}
