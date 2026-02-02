use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  ConfigureSettings,

  InviteUsers,
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
      Role::Owner => vec![Permission::ConfigureSettings, Permission::InviteUsers],
      Role::Admin => vec![Permission::InviteUsers],
      Role::Undefined => vec![],
    }
  }

  pub fn has_permission(&self, perm: Permission) -> bool {
    self.permissions().contains(&perm)
  }

  pub fn can_assign_role(&self, target_role: Role) -> bool {
    let my_perms = self.permissions();
    let target_perms = target_role.permissions();

    target_perms.iter().all(|p| my_perms.contains(p))
  }
}
