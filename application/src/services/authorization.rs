use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use domain::UserId;

/// Service for checking whether a user has a specific permission in a given scope.
///
/// Permission resolution works as follows:
/// 1. All roles assigned to the user (via `user_roles`) are collected.
/// 2. For each role, its permissions are resolved recursively via role inheritance.
/// 3. A permission match is found if the `(action, subject)` pair exists in any of
///    those role_permissions entries where:
///    - `scope_kind = 'global'` (applies everywhere), OR
///    - `scope_kind = $scope_kind AND scope_id = $scope_id` (applies to this specific resource), OR
///    - `scope_kind = $scope_kind AND scope_id IS NULL` (applies to all resources of this kind)
#[derive(Clone)]
pub struct AuthorizationService {
  pool: PgPool,
}

impl AuthorizationService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Returns `true` if the user has permission to perform `action` on `subject`.
  ///
  /// `scope` is `None` for a global check, or `Some((scope_kind, scope_id))` for a
  /// resource-scoped check (e.g. `Some(("shop", shop_uuid))`).
  pub async fn has_permission(
    &self,
    user_id: UserId,
    action: &str,
    subject: &str,
    scope: Option<(&str, Uuid)>,
  ) -> AppResult<bool> {
    let (scope_kind, scope_id): (&str, Option<Uuid>) = match scope {
      None => ("global", None),
      Some((kind, id)) => (kind, Some(id)),
    };

    // Resolve all permissions for all roles assigned to the user, including inherited
    // roles, in a single query.
    let count: i64 = sqlx::query_scalar(
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
      JOIN permissions p ON p.id = rp.permission_id
      WHERE rp.role_id IN (SELECT id FROM role_hierarchy)
        AND p.action = $2
        AND p.subject = $3
        AND (
          rp.scope_kind = 'global'
          OR (rp.scope_kind = $4 AND (rp.scope_id = $5 OR rp.scope_id IS NULL))
        )
      "#,
    )
    .bind(user_id.into_inner())
    .bind(action)
    .bind(subject)
    .bind(scope_kind)
    .bind(scope_id)
    .fetch_one(&self.pool)
    .await?;

    Ok(count > 0)
  }

  /// Enforces that the user has permission to perform `action` on `subject` globally.
  ///
  /// Returns `Err(AppError::PermissionDenied { ... })` if the check fails.
  pub async fn require(
    &self,
    user_id: UserId,
    action: &str,
    subject: &str,
  ) -> AppResult<()> {
    if !self.has_permission(user_id, action, subject, None).await? {
      return Err(AppError::PermissionDenied {
        action: action.to_string(),
        subject: subject.to_string(),
        scope: "global".to_string(),
      });
    }
    Ok(())
  }

  /// Enforces that the user has permission to perform `action` on `subject` in a
  /// specific resource scope (e.g. a particular shop or event).
  ///
  /// Permission is granted if the user has a matching global permission **or** a
  /// matching scoped permission for the given resource.
  pub async fn require_scoped(
    &self,
    user_id: UserId,
    action: &str,
    subject: &str,
    scope_kind: &str,
    scope_id: Uuid,
  ) -> AppResult<()> {
    if !self
      .has_permission(user_id, action, subject, Some((scope_kind, scope_id)))
      .await?
    {
      return Err(AppError::PermissionDenied {
        action: action.to_string(),
        subject: subject.to_string(),
        scope: format!("{}:{}", scope_kind, scope_id),
      });
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Ensure the service can be constructed with a pool reference (compile-time check).
  #[allow(dead_code)]
  fn assert_send_sync() {
    fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<AuthorizationService>();
  }
}
