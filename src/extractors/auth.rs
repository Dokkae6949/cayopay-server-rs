use std::collections::HashSet;
use std::ops::Deref;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{permission::Permission, User};
use crate::services::{authorization::AuthorizationService, session};
use crate::state::AppState;

const GLOBAL_SCOPE: &str = "global";

/// Combined authentication + authorisation extractor.
///
/// Pulling `Auth` from a handler extracts the authenticated caller *and*
/// pre-loads their full set of global permissions in a single DB round-trip,
/// so handlers can call the synchronous [`Auth::require`] instead of wiring
/// `authz_service` + `user_id` through every call-site.
pub struct Auth {
  pub user: User,
  permissions: HashSet<Permission>,
  authz: AuthorizationService,
}

impl Auth {
  /// Enforces that the caller holds the given global permission.
  pub fn require(&self, permission: Permission) -> AppResult<()> {
    if self.permissions.contains(&permission) {
      Ok(())
    } else {
      Err(AppError::PermissionDenied {
        permission: permission.to_string(),
        scope: GLOBAL_SCOPE.to_string(),
      })
    }
  }

  /// Enforces that the caller holds the given permission scoped to a specific resource.
  pub async fn require_scoped(
    &self,
    permission: Permission,
    scope_kind: &str,
    scope_id: Uuid,
  ) -> AppResult<()> {
    self
      .authz
      .require_scoped(self.user.id, permission, scope_kind, scope_id)
      .await
  }
}

impl Deref for Auth {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.user
  }
}

#[async_trait]
impl FromRequestParts<AppState> for Auth {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let jar = parts
      .extract::<CookieJar>()
      .await
      .map_err(|_| AppError::Authentication)?;

    let token = jar
      .get(&state.config.session_cookie_name)
      .ok_or(AppError::Authentication)?
      .value()
      .to_owned();

    let user = session::authenticate(&state.pool, &token)
      .await?
      .ok_or(AppError::Authentication)?;

    let permissions = state
      .authz_service
      .load_global_permissions(user.id)
      .await?;

    Ok(Auth {
      user,
      permissions,
      authz: state.authz_service.clone(),
    })
  }
}
