use uuid::Uuid;

use crate::models::ShopId;

// ---------------------------------------------------------------------------
// Permission string constants
// ---------------------------------------------------------------------------

/// Configure system-wide settings.
pub const CONFIGURE_SETTINGS: &str = "settings.configure";
/// Send an invite to a new user.
pub const SEND_INVITE: &str = "invite.send";
/// View all existing invites.
pub const VIEW_INVITE: &str = "invite.view";
/// Remove a user from the system.
pub const REMOVE_USER: &str = "user.remove";
/// Read user details.
pub const READ_USER: &str = "user.read";
/// Remove a guest from the system.
pub const REMOVE_GUEST: &str = "guest.remove";
/// Read guest details.
pub const READ_GUEST: &str = "guest.read";

// ---------------------------------------------------------------------------
// PermissionDef – catalogue for the frontend
// ---------------------------------------------------------------------------

/// Static metadata about a single permission, returned by `GET /api/permissions`.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct PermissionDef {
  /// The permission string stored in the database (e.g. `"settings.configure"`).
  pub name: &'static str,
  /// Human-readable description.
  pub description: &'static str,
  /// Which resource type this permission targets: `"global"` or `"shop"`.
  pub resource_type: &'static str,
}

/// All permissions recognised by the system.
pub const PERMISSIONS: &[PermissionDef] = &[
  PermissionDef {
    name: CONFIGURE_SETTINGS,
    description: "Configure system-wide settings.",
    resource_type: "global",
  },
  PermissionDef {
    name: SEND_INVITE,
    description: "Send an invite to a new user.",
    resource_type: "global",
  },
  PermissionDef {
    name: VIEW_INVITE,
    description: "View all existing invites.",
    resource_type: "global",
  },
  PermissionDef {
    name: REMOVE_USER,
    description: "Remove a user from the system.",
    resource_type: "global",
  },
  PermissionDef {
    name: READ_USER,
    description: "Read user details.",
    resource_type: "global",
  },
  PermissionDef {
    name: REMOVE_GUEST,
    description: "Remove a guest from the system.",
    resource_type: "global",
  },
  PermissionDef {
    name: READ_GUEST,
    description: "Read guest details.",
    resource_type: "global",
  },
];

// ---------------------------------------------------------------------------
// Resource – type-safe resource context for permission checks
// ---------------------------------------------------------------------------

/// The resource context for a permission check.
///
/// - `Global` – system-wide, no specific resource.
/// - `Shop(Some(id))` – the permission applies only to this specific shop.
/// - `Shop(None)` – the permission applies to *any* shop (wildcard).
#[derive(Debug, Clone)]
pub enum Resource {
  /// No resource context; the permission is global.
  Global,
  /// A shop resource, or `None` to match any shop.
  Shop(Option<ShopId>),
}

impl Resource {
  /// The `resource_type` column value stored in `granted_permissions`.
  pub fn resource_type(&self) -> &'static str {
    match self {
      Resource::Global => "global",
      Resource::Shop(_) => "shop",
    }
  }

  /// The `resource_id` column value; `None` means "any resource of this type".
  pub fn resource_id(&self) -> Option<Uuid> {
    match self {
      Resource::Global => None,
      Resource::Shop(id) => id.map(|i| i.into_inner()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_permissions_have_nonempty_names() {
    for p in PERMISSIONS {
      assert!(!p.name.is_empty());
      assert!(!p.description.is_empty());
      assert!(p.resource_type == "global" || p.resource_type == "shop");
    }
  }

  #[test]
  fn resource_type_strings() {
    assert_eq!(Resource::Global.resource_type(), "global");
    assert_eq!(Resource::Shop(None).resource_type(), "shop");
  }
}
