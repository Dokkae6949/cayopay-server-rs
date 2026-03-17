use crate::extractor::Authn;
use application::{error::AppResult, services::AuthorizationService, state::AppState};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use domain::{models::permission::Permission, User, UserId};
use uuid::Uuid;

use crate::error::ApiError;

/// `Authz` is an Axum extractor that requires an authenticated session and provides
/// type-safe, runtime permission checking close to the protected operation.
///
/// Permissions are defined in code as the [`Permission`] enum.  Roles (and which
/// permissions each role holds) are managed at runtime via the database.
///
/// # Usage in handlers
///
/// ```rust,ignore
/// // Global permission check
/// async fn my_handler(authz: Authz) -> impl IntoResponse {
///   authz.require(Permission::SendInvite).await?;
///   // ... protected code
/// }
///
/// // Scoped permission check
/// async fn shop_handler(authz: Authz, Path(shop_id): Path<Uuid>) -> impl IntoResponse {
///   authz.require_scoped(Permission::ReadUser, "shop", shop_id).await?;
///   // ... protected code specific to this shop
/// }
/// ```
pub struct Authz {
  pub user: User,
  authz_service: AuthorizationService,
}

impl Authz {
  /// Returns the ID of the authenticated user.
  pub fn user_id(&self) -> UserId {
    self.user.id
  }

  /// Checks that the user has a global permission.
  ///
  /// Permission is resolved by traversing all roles assigned to this user
  /// (including inherited roles) and checking for a matching permission entry
  /// with `scope_kind IS NULL`.
  pub async fn require(&self, permission: Permission) -> AppResult<()> {
    self.authz_service.require(self.user.id, permission).await
  }

  /// Checks that the user has a permission in a specific resource scope.
  ///
  /// A global version of the same permission (scope_kind IS NULL) also satisfies
  /// this check, so admins with global access can act on any specific resource.
  pub async fn require_scoped(
    &self,
    permission: Permission,
    scope_kind: &str,
    scope_id: Uuid,
  ) -> AppResult<()> {
    self
      .authz_service
      .require_scoped(self.user.id, permission, scope_kind, scope_id)
      .await
  }

  /// Returns `true` if the user has the global permission, `false` otherwise.
  pub async fn has_permission(&self, permission: Permission) -> bool {
    self
      .authz_service
      .has_permission(self.user.id, permission, None)
      .await
      .unwrap_or(false)
  }

  /// Returns `true` if the user has the permission in the given resource scope.
  pub async fn has_permission_scoped(
    &self,
    permission: Permission,
    scope_kind: &str,
    scope_id: Uuid,
  ) -> bool {
    self
      .authz_service
      .has_permission(self.user.id, permission, Some((scope_kind, scope_id)))
      .await
      .unwrap_or(false)
  }
}

#[async_trait]
impl FromRequestParts<AppState> for Authz {
  type Rejection = ApiError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let user = Authn::from_request_parts(parts, state).await?.0;
    Ok(Authz {
      user,
      authz_service: state.authz_service.clone(),
    })
  }
}
