use std::ops::Deref;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;

use crate::error::{AppError, AppResult};
use crate::models::{Resource, ShopId, User};
use crate::services::{authorization::AuthorizationService, session};
use crate::state::AppState;

/// Combined authentication + authorisation extractor.
///
/// Extracting `Authz` from a handler authenticates the caller from the session
/// cookie and provides async permission-checking methods that query the
/// `granted_permissions` table on demand.
///
/// # Usage
/// ```rust,ignore
/// async fn my_handler(authz: Authz) -> AppResult<()> {
///     authz.require_global("settings.configure").await?;
///     authz.require_shop("product.create", Some(shop_id)).await?;
///     // authz also derefs to the authenticated User
///     println!("{}", authz.first_name);
///     Ok(())
/// }
/// ```
pub struct Authz {
  pub user: User,
  authz: AuthorizationService,
}

impl Authz {
  /// Enforces a **global** permission (no resource context).
  pub async fn require_global(&self, permission: &str) -> AppResult<()> {
    self.authz.require_global(self.user.id, permission).await
  }

  /// Enforces a permission for the given [`Resource`] context.
  pub async fn require_resource(&self, permission: &str, resource: Resource) -> AppResult<()> {
    self.authz.require_resource(self.user.id, permission, resource).await
  }

  /// Convenience: enforces a shop-scoped permission.
  ///
  /// - `shop_id = Some(id)` – checks for that specific shop (wildcard also satisfies).
  /// - `shop_id = None`     – checks for an "any shop" wildcard grant.
  pub async fn require_shop(&self, permission: &str, shop_id: Option<ShopId>) -> AppResult<()> {
    self.authz.require_shop(self.user.id, permission, shop_id).await
  }
}

impl Deref for Authz {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.user
  }
}

#[async_trait]
impl FromRequestParts<AppState> for Authz {
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

    Ok(Authz {
      user,
      authz: state.authz_service.clone(),
    })
  }
}
