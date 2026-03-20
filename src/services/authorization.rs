use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use crate::models::permission::{GlobalPermission, Resource, ShopPermission};
use crate::models::{ShopId, UserId};

/// Service for checking whether a user has been directly granted a permission.
///
/// Permission resolution uses the `granted_permissions` table:
///
/// ```text
/// granted_permissions (user_id, resource_type, resource_id, permission)
/// ```
///
/// - `resource_type = 'global'`, `resource_id IS NULL` → system-wide permission.
/// - `resource_type = 'shop'`,   `resource_id IS NULL` → permission on *any* shop.
/// - `resource_type = 'shop'`,   `resource_id = <uuid>` → permission on that specific shop.
///
/// There is no role inheritance; each row is a direct grant.
#[derive(Clone)]
pub struct AuthorizationService {
  pool: PgPool,
}

impl AuthorizationService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Returns `true` if the user holds the permission for the given resource context.
  ///
  /// For a `Shop(Some(id))` check, a wildcard grant (`resource_id IS NULL`)
  /// also satisfies the check.
  pub async fn has_permission(
    &self,
    user_id: UserId,
    permission: &str,
    resource: &Resource,
  ) -> AppResult<bool> {
    let resource_type = resource.resource_type();
    let resource_id = resource.resource_id();

    let count: i64 = match resource_id {
      None => {
        // Global check OR "any resource" wildcard check: exact NULL match.
        sqlx::query_scalar(
          r#"
          SELECT COUNT(*)
          FROM granted_permissions
          WHERE user_id       = $1
            AND permission    = $2
            AND resource_type = $3
            AND resource_id   IS NULL
          "#,
        )
        .bind(user_id.into_inner())
        .bind(permission)
        .bind(resource_type)
        .fetch_one(&self.pool)
        .await?
      }
      Some(id) => {
        // Specific resource: match the exact resource_id OR a wildcard (NULL) grant.
        sqlx::query_scalar(
          r#"
          SELECT COUNT(*)
          FROM granted_permissions
          WHERE user_id       = $1
            AND permission    = $2
            AND resource_type = $3
            AND (resource_id = $4 OR resource_id IS NULL)
          "#,
        )
        .bind(user_id.into_inner())
        .bind(permission)
        .bind(resource_type)
        .bind(id)
        .fetch_one(&self.pool)
        .await?
      }
    };

    Ok(count > 0)
  }

  /// Enforces a **global** permission (no resource context).
  ///
  /// Returns `Err(AppError::PermissionDenied)` if the user does not hold the permission.
  pub async fn require_global(&self, user_id: UserId, permission: &str) -> AppResult<()> {
    if !self.has_permission(user_id, permission, &Resource::Global).await? {
      return Err(AppError::PermissionDenied {
        permission: permission.to_string(),
        scope: "global".to_string(),
      });
    }
    Ok(())
  }

  /// Enforces a permission for the given [`Resource`] context.
  ///
  /// For `Resource::Shop(Some(id))` a wildcard shop grant also satisfies the check.
  pub async fn require_resource(
    &self,
    user_id: UserId,
    permission: &str,
    resource: Resource,
  ) -> AppResult<()> {
    if !self.has_permission(user_id, permission, &resource).await? {
      let scope = match &resource {
        Resource::Global => "global".to_string(),
        Resource::Shop(None) => "shop:any".to_string(),
        Resource::Shop(Some(id)) => format!("shop:{}", id),
      };
      return Err(AppError::PermissionDenied {
        permission: permission.to_string(),
        scope,
      });
    }
    Ok(())
  }

  /// Convenience wrapper: enforces a shop-scoped permission for a specific shop.
  ///
  /// A wildcard grant (`resource_id IS NULL`) also satisfies the check.
  pub async fn require_shop(
    &self,
    user_id: UserId,
    permission: &str,
    shop_id: ShopId,
  ) -> AppResult<()> {
    self.require_resource(user_id, permission, Resource::Shop(Some(shop_id))).await
  }
}

// ---------------------------------------------------------------------------
// PermissionEngine – typed, scope-specific permission checking
// ---------------------------------------------------------------------------

/// A scoped permission engine that checks permissions within a fixed resource context.
///
/// The associated type [`Permission`][PermissionEngine::Permission] enforces that
/// only permissions belonging to the correct scope can be used with each engine —
/// you cannot pass a [`ShopPermission`] to a [`GlobalEngine`] or vice versa.
///
/// ```rust,ignore
/// authz.global().require(GlobalPermission::ReadUser).await?;
/// authz.global().require_all(&[GlobalPermission::ReadUser, GlobalPermission::RemoveUser]).await?;
/// authz.shop(shop_id).require(ShopPermission::CreateProduct).await?;
/// ```
pub trait PermissionEngine {
  /// The permission type accepted by this engine.
  type Permission: Copy;

  /// Require the caller to hold exactly this permission.
  ///
  /// Returns `Err(AppError::PermissionDenied)` when the permission is absent.
  async fn require(&self, permission: Self::Permission) -> AppResult<()>;

  /// Require the caller to hold **at least one** of the given permissions.
  ///
  /// Returns `Err(AppError::PermissionDenied)` only when none are held.
  async fn require_any(&self, permissions: &[Self::Permission]) -> AppResult<()>;

  /// Require the caller to hold **all** of the given permissions.
  ///
  /// Returns `Err(AppError::PermissionDenied)` on the first missing permission.
  async fn require_all(&self, permissions: &[Self::Permission]) -> AppResult<()>;
}

// ---------------------------------------------------------------------------
// GlobalEngine
// ---------------------------------------------------------------------------

/// Permission engine for global (system-wide) permission checks.
///
/// Obtain one via [`Authz::global`][crate::extractors::Authz::global].
pub struct GlobalEngine {
  pub(crate) user_id: UserId,
  pub(crate) authz: AuthorizationService,
}

impl PermissionEngine for GlobalEngine {
  type Permission = GlobalPermission;

  async fn require(&self, permission: GlobalPermission) -> AppResult<()> {
    self.authz.require_global(self.user_id, permission.as_str()).await
  }

  async fn require_any(&self, permissions: &[GlobalPermission]) -> AppResult<()> {
    for &permission in permissions {
      if self.authz.has_permission(self.user_id, permission.as_str(), &Resource::Global).await? {
        return Ok(());
      }
    }
    let names: Vec<&str> = permissions.iter().map(|p| p.as_str()).collect();
    Err(AppError::PermissionDenied {
      permission: format!("any of [{}]", names.join(", ")),
      scope: "global".to_string(),
    })
  }

  async fn require_all(&self, permissions: &[GlobalPermission]) -> AppResult<()> {
    for &permission in permissions {
      self.authz.require_global(self.user_id, permission.as_str()).await?;
    }
    Ok(())
  }
}

// ---------------------------------------------------------------------------
// ShopEngine
// ---------------------------------------------------------------------------

/// Permission engine for shop-scoped permission checks.
///
/// Always requires a specific [`ShopId`] — a wildcard grant (`resource_id IS NULL`)
/// on that shop type also satisfies any check.
///
/// Obtain one via [`Authz::shop`][crate::extractors::Authz::shop].
pub struct ShopEngine {
  pub(crate) user_id: UserId,
  pub(crate) shop_id: ShopId,
  pub(crate) authz: AuthorizationService,
}

impl ShopEngine {
  fn scope(&self) -> String {
    format!("shop:{}", self.shop_id)
  }
}

impl PermissionEngine for ShopEngine {
  type Permission = ShopPermission;

  async fn require(&self, permission: ShopPermission) -> AppResult<()> {
    self.authz.require_shop(self.user_id, permission.as_str(), self.shop_id).await
  }

  async fn require_any(&self, permissions: &[ShopPermission]) -> AppResult<()> {
    let resource = Resource::Shop(Some(self.shop_id));
    for &permission in permissions {
      if self.authz.has_permission(self.user_id, permission.as_str(), &resource).await? {
        return Ok(());
      }
    }
    let names: Vec<&str> = permissions.iter().map(|p| p.as_str()).collect();
    Err(AppError::PermissionDenied {
      permission: format!("any of [{}]", names.join(", ")),
      scope: self.scope(),
    })
  }

  async fn require_all(&self, permissions: &[ShopPermission]) -> AppResult<()> {
    for &permission in permissions {
      self.authz.require_shop(self.user_id, permission.as_str(), self.shop_id).await?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[allow(dead_code)]
  fn assert_send_sync() {
    fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<AuthorizationService>();
  }
}
