use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use domain::models::permission::Permission;
use domain::UserId;

/// Service for checking whether a user has a specific permission in a given scope.
///
/// Permission resolution works as follows:
/// 1. All roles assigned to the user (via `user_roles`) are collected.
/// 2. For each role, its permissions are resolved recursively via role inheritance.
/// 3. A permission match is found if the permission code exists in any of those
///    `role_permissions` entries where:
///    - `scope_kind IS NULL` (global — applies everywhere), OR
///    - `scope_kind = $scope_kind AND (scope_id = $scope_id OR scope_id IS NULL)`
///      (scoped to this specific resource kind, or all resources of that kind)
#[derive(Clone)]
pub struct AuthorizationService {
  pool: PgPool,
}

impl AuthorizationService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Returns `true` if the user has the given permission.
  ///
  /// `scope` is `None` for a global check, or `Some((scope_kind, scope_id))` for a
  /// resource-scoped check (e.g. `Some(("shop", shop_uuid))`).
  pub async fn has_permission(
    &self,
    user_id: UserId,
    permission: Permission,
    scope: Option<(&str, Uuid)>,
  ) -> AppResult<bool> {
    let count: i64 = match scope {
      None => {
        // Global check: permission must have a NULL scope_kind entry.
        sqlx::query_scalar(
          r#"
          WITH RECURSIVE role_hierarchy AS (
            SELECT ur.role_id AS id
            FROM user_roles ur
            WHERE ur.user_id = $1

            UNION

            SELECT r.inherited_from_role_id
            FROM roles r
            JOIN role_hierarchy rh ON r.id = rh.id
            WHERE r.inherited_from_role_id IS NOT NULL
          )
          SELECT COUNT(*)
          FROM role_permissions rp
          WHERE rp.role_id IN (SELECT id FROM role_hierarchy)
            AND rp.permission = $2
            AND rp.scope_kind IS NULL
          "#,
        )
        .bind(user_id.into_inner())
        .bind(permission.as_str())
        .fetch_one(&self.pool)
        .await?
      }
      Some((scope_kind, scope_id)) => {
        // Scoped check: a global entry (scope_kind IS NULL) also satisfies the check.
        sqlx::query_scalar(
          r#"
          WITH RECURSIVE role_hierarchy AS (
            SELECT ur.role_id AS id
            FROM user_roles ur
            WHERE ur.user_id = $1

            UNION

            SELECT r.inherited_from_role_id
            FROM roles r
            JOIN role_hierarchy rh ON r.id = rh.id
            WHERE r.inherited_from_role_id IS NOT NULL
          )
          SELECT COUNT(*)
          FROM role_permissions rp
          WHERE rp.role_id IN (SELECT id FROM role_hierarchy)
            AND rp.permission = $2
            AND (
              rp.scope_kind IS NULL
              OR (rp.scope_kind = $3 AND (rp.scope_id = $4 OR rp.scope_id IS NULL))
            )
          "#,
        )
        .bind(user_id.into_inner())
        .bind(permission.as_str())
        .bind(scope_kind)
        .bind(scope_id)
        .fetch_one(&self.pool)
        .await?
      }
    };

    Ok(count > 0)
  }

  /// Enforces that the user has the given global permission.
  ///
  /// Returns `Err(AppError::PermissionDenied)` if the check fails.
  pub async fn require(&self, user_id: UserId, permission: Permission) -> AppResult<()> {
    if !self.has_permission(user_id, permission, None).await? {
      return Err(AppError::PermissionDenied {
        permission: permission.to_string(),
        scope: "global".to_string(),
      });
    }
    Ok(())
  }

  /// Enforces that the user has the given permission in a specific resource scope.
  ///
  /// A global version of the same permission (scope_kind IS NULL) also satisfies
  /// this check, so admins with global access can act on any specific resource.
  pub async fn require_scoped(
    &self,
    user_id: UserId,
    permission: Permission,
    scope_kind: &str,
    scope_id: Uuid,
  ) -> AppResult<()> {
    if !self
      .has_permission(user_id, permission, Some((scope_kind, scope_id)))
      .await?
    {
      return Err(AppError::PermissionDenied {
        permission: permission.to_string(),
        scope: format!("{}:{}", scope_kind, scope_id),
      });
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
