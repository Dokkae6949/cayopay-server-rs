use uuid::Uuid;

use crate::models::ShopId;

// ---------------------------------------------------------------------------
// GlobalPermission – type-safe global (system-wide) permissions
// ---------------------------------------------------------------------------

/// All system-wide (global) permissions recognised by the authorization engine.
///
/// Use this enum with [`GlobalEngine`][crate::services::authorization::GlobalEngine]:
/// ```rust,ignore
/// authz.global().require(GlobalPermission::ReadUser).await?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalPermission {
  /// Configure system-wide settings.
  ConfigureSettings,
  /// Send an invite to a new user.
  SendInvite,
  /// View all existing invites.
  ViewInvite,
  /// Remove a user from the system.
  RemoveUser,
  /// Read user details.
  ReadUser,
  /// Remove a guest from the system.
  RemoveGuest,
  /// Read guest details.
  ReadGuest,
}

impl GlobalPermission {
  /// The permission string stored in the database.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::ConfigureSettings => "settings.configure",
      Self::SendInvite => "invite.send",
      Self::ViewInvite => "invite.view",
      Self::RemoveUser => "user.remove",
      Self::ReadUser => "user.read",
      Self::RemoveGuest => "guest.remove",
      Self::ReadGuest => "guest.read",
    }
  }
}

// ---------------------------------------------------------------------------
// ShopPermission – type-safe shop-scoped permissions
// ---------------------------------------------------------------------------

/// All shop-scoped permissions recognised by the authorization engine.
///
/// Use this enum with [`ShopEngine`][crate::services::authorization::ShopEngine]:
/// ```rust,ignore
/// authz.shop(shop_id).require(ShopPermission::CreateProduct).await?;
/// ```
///
/// # Note
/// This enum is currently empty (no shop-scoped permissions have been defined yet).
/// It exists as a typed placeholder so that `ShopEngine` is correctly scoped to its
/// own permission type from the start.  Add variants here when shop permissions are
/// introduced; the compiler will then require all `ShopEngine` call-sites to be
/// updated to use the correct type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopPermission {}

impl ShopPermission {
  /// The permission string stored in the database.
  ///
  /// This method can never be called because `ShopPermission` is uninhabited —
  /// Rust allows an empty `match self {}` for such types.
  pub const fn as_str(self) -> &'static str {
    match self {}
  }
}

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
    name: GlobalPermission::ConfigureSettings.as_str(),
    description: "Configure system-wide settings.",
    resource_type: "global",
  },
  PermissionDef {
    name: GlobalPermission::SendInvite.as_str(),
    description: "Send an invite to a new user.",
    resource_type: "global",
  },
  PermissionDef {
    name: GlobalPermission::ViewInvite.as_str(),
    description: "View all existing invites.",
    resource_type: "global",
  },
  PermissionDef {
    name: GlobalPermission::RemoveUser.as_str(),
    description: "Remove a user from the system.",
    resource_type: "global",
  },
  PermissionDef {
    name: GlobalPermission::ReadUser.as_str(),
    description: "Read user details.",
    resource_type: "global",
  },
  PermissionDef {
    name: GlobalPermission::RemoveGuest.as_str(),
    description: "Remove a guest from the system.",
    resource_type: "global",
  },
  PermissionDef {
    name: GlobalPermission::ReadGuest.as_str(),
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
  fn global_permission_as_str() {
    assert_eq!(GlobalPermission::ConfigureSettings.as_str(), "settings.configure");
    assert_eq!(GlobalPermission::SendInvite.as_str(), "invite.send");
    assert_eq!(GlobalPermission::ViewInvite.as_str(), "invite.view");
    assert_eq!(GlobalPermission::RemoveUser.as_str(), "user.remove");
    assert_eq!(GlobalPermission::ReadUser.as_str(), "user.read");
    assert_eq!(GlobalPermission::RemoveGuest.as_str(), "guest.remove");
    assert_eq!(GlobalPermission::ReadGuest.as_str(), "guest.read");
  }

  #[test]
  fn resource_type_strings() {
    assert_eq!(Resource::Global.resource_type(), "global");
    assert_eq!(Resource::Shop(None).resource_type(), "shop");
  }
}
