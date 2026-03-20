use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use crate::models::permission::Resource;
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

  /// Convenience wrapper: enforces a shop-scoped permission.
  ///
  /// - `shop_id = Some(id)` → checks for `shop:<id>` (wildcard grant also satisfies).
  /// - `shop_id = None`     → checks for a wildcard `shop:any` grant.
  pub async fn require_shop(
    &self,
    user_id: UserId,
    permission: &str,
    shop_id: Option<crate::models::ShopId>,
  ) -> AppResult<()> {
    self.require_resource(user_id, permission, Resource::Shop(shop_id)).await
  }
}

// ---------------------------------------------------------------------------
// PermissionEngine – typed, scope-specific permission checking
// ---------------------------------------------------------------------------

/// A scoped permission engine that checks permissions within a fixed resource context.
///
/// Each implementor is bound to a particular user and a particular scope (e.g.
/// "global" or "shop:<id>"), so callers can express requirements concisely:
///
/// ```rust,ignore
/// authz.global().require(READ_USER).await?;
/// authz.global().require_all(&[READ_USER, REMOVE_USER]).await?;
/// authz.shop(Some(shop_id)).require("product.create").await?;
/// ```
pub trait PermissionEngine {
  /// Require the caller to hold exactly this permission.
  ///
  /// Returns `Err(AppError::PermissionDenied)` when the permission is absent.
  async fn require(&self, permission: &str) -> AppResult<()>;

  /// Require the caller to hold **at least one** of the given permissions.
  ///
  /// Returns `Err(AppError::PermissionDenied)` only when none are held.
  async fn require_any(&self, permissions: &[&str]) -> AppResult<()>;

  /// Require the caller to hold **all** of the given permissions.
  ///
  /// Returns `Err(AppError::PermissionDenied)` on the first missing permission.
  async fn require_all(&self, permissions: &[&str]) -> AppResult<()>;
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
  async fn require(&self, permission: &str) -> AppResult<()> {
    self.authz.require_global(self.user_id, permission).await
  }

  async fn require_any(&self, permissions: &[&str]) -> AppResult<()> {
    for &permission in permissions {
      if self.authz.has_permission(self.user_id, permission, &Resource::Global).await? {
        return Ok(());
      }
    }
    Err(AppError::PermissionDenied {
      permission: format!("any of [{}]", permissions.join(", ")),
      scope: "global".to_string(),
    })
  }

  async fn require_all(&self, permissions: &[&str]) -> AppResult<()> {
    for &permission in permissions {
      self.authz.require_global(self.user_id, permission).await?;
    }
    Ok(())
  }
}

// ---------------------------------------------------------------------------
// ShopEngine
// ---------------------------------------------------------------------------

/// Permission engine for shop-scoped permission checks.
///
/// - `shop_id = Some(id)` – checks permissions for that specific shop
///   (a wildcard grant also satisfies the check).
/// - `shop_id = None` – checks for a "any shop" wildcard grant.
///
/// Obtain one via [`Authz::shop`][crate::extractors::Authz::shop].
pub struct ShopEngine {
  pub(crate) user_id: UserId,
  pub(crate) shop_id: Option<ShopId>,
  pub(crate) authz: AuthorizationService,
}

impl ShopEngine {
  fn scope(&self) -> String {
    match self.shop_id {
      None => "shop:any".to_string(),
      Some(id) => format!("shop:{id}"),
    }
  }
}

impl PermissionEngine for ShopEngine {
  async fn require(&self, permission: &str) -> AppResult<()> {
    self.authz.require_shop(self.user_id, permission, self.shop_id).await
  }

  async fn require_any(&self, permissions: &[&str]) -> AppResult<()> {
    let resource = Resource::Shop(self.shop_id);
    for &permission in permissions {
      if self.authz.has_permission(self.user_id, permission, &resource).await? {
        return Ok(());
      }
    }
    Err(AppError::PermissionDenied {
      permission: format!("any of [{}]", permissions.join(", ")),
      scope: self.scope(),
    })
  }

  async fn require_all(&self, permissions: &[&str]) -> AppResult<()> {
    for &permission in permissions {
      self.authz.require_shop(self.user_id, permission, self.shop_id).await?;
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
