use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{permission::Permission, UserId};

/// Service for checking whether a user has a specific permission in a given scope.
///
/// Permission resolution:
/// - Roles are flat collections of permissions (no inheritance).
/// - A user can hold multiple roles (`user_roles`).
/// - Each `role_permissions` row ties a permission to a role with an optional scope:
///   - `scope_kind IS NULL` → global (applies everywhere)
///   - `scope_kind = "shop"`, `scope_id = <uuid>` → scoped to that specific resource
///   - `scope_kind = "shop"`, `scope_id IS NULL` → all resources of that kind
#[derive(Clone)]
pub struct AuthorizationService {
  pool: PgPool,
}

impl AuthorizationService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Loads every global permission (`scope_kind IS NULL`) currently held by the user.
  /// Unknown string codes in the DB (e.g. from a stale migration) are skipped with a warning.
  pub async fn load_global_permissions(&self, user_id: UserId) -> AppResult<HashSet<Permission>> {
    let codes: Vec<String> = sqlx::query_scalar(
      r#"
      SELECT DISTINCT rp.permission
      FROM user_roles ur
      JOIN role_permissions rp ON rp.role_id = ur.role_id
      WHERE ur.user_id = $1
        AND rp.scope_kind IS NULL
      "#,
    )
    .bind(user_id.into_inner())
    .fetch_all(&self.pool)
    .await?;

    Ok(codes
      .iter()
      .filter_map(|c| {
        let p = Permission::from_code(c);
        if p.is_none() {
          tracing::warn!(code = %c, "Skipping unknown permission code from DB (stale migration?)");
        }
        p
      })
      .collect())
  }

  /// Returns `true` if the user has the given permission.
  ///
  /// `scope` is `None` for a global check, or `Some((scope_kind, scope_id))` for a
  /// resource-scoped check (e.g. `Some(("shop", shop_uuid))`).
  ///
  /// A global entry (`scope_kind IS NULL`) satisfies any scoped check.
  pub async fn has_permission(
    &self,
    user_id: UserId,
    permission: Permission,
    scope: Option<(&str, Uuid)>,
  ) -> AppResult<bool> {
    let count: i64 = match scope {
      None => {
        sqlx::query_scalar(
          r#"
          SELECT COUNT(*)
          FROM user_roles ur
          JOIN role_permissions rp ON rp.role_id = ur.role_id
          WHERE ur.user_id = $1
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
        sqlx::query_scalar(
          r#"
          SELECT COUNT(*)
          FROM user_roles ur
          JOIN role_permissions rp ON rp.role_id = ur.role_id
          WHERE ur.user_id = $1
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
  /// A global version of the same permission (`scope_kind IS NULL`) also satisfies
  /// this check, so users with global access can act on any specific resource.
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
