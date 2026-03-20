use std::ops::Deref;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;

use crate::error::AppError;
use crate::models::{ShopId, User};
use crate::services::{
  authorization::{AuthorizationService, GlobalEngine, ShopEngine},
  session,
};
use crate::state::AppState;

/// Combined authentication + authorisation extractor.
///
/// Extracting `Authz` from a handler authenticates the caller from the session
/// cookie and exposes typed, scoped permission engines for clean permission checks.
///
/// # Usage
/// ```rust,ignore
/// async fn my_handler(authz: Authz) -> AppResult<()> {
///     // Single permission
///     authz.global().require("settings.configure").await?;
///     // Any-of check
///     authz.global().require_any(&["invite.send", "invite.view"]).await?;
///     // All-of check
///     authz.global().require_all(&["user.read", "user.remove"]).await?;
///     // Shop-scoped check
///     authz.shop(Some(shop_id)).require("product.create").await?;
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
  /// Returns a [`GlobalEngine`] scoped to the authenticated user for
  /// system-wide permission checks.
  pub fn global(&self) -> GlobalEngine {
    GlobalEngine { user_id: self.user.id, authz: self.authz.clone() }
  }

  /// Returns a [`ShopEngine`] scoped to the authenticated user for
  /// shop-scoped permission checks.
  ///
  /// - `shop_id = Some(id)` – checks permissions for that specific shop.
  /// - `shop_id = None` – checks for a wildcard "any shop" grant.
  pub fn shop(&self, shop_id: Option<ShopId>) -> ShopEngine {
    ShopEngine { user_id: self.user.id, shop_id, authz: self.authz.clone() }
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

/// Extension trait that adds scoped permission-engine accessors to any type
/// that holds an [`Authz`].
///
/// This is automatically implemented for [`Authz`] itself.  Third-party
/// middleware wrappers can implement it for their own types.
pub trait AuthzExt {
  /// Returns a [`GlobalEngine`] for system-wide permission checks.
  fn global(&self) -> GlobalEngine;

  /// Returns a [`ShopEngine`] for shop-scoped permission checks.
  fn shop(&self, shop_id: Option<ShopId>) -> ShopEngine;
}

impl AuthzExt for Authz {
  fn global(&self) -> GlobalEngine {
    Authz::global(self)
  }

  fn shop(&self, shop_id: Option<ShopId>) -> ShopEngine {
    Authz::shop(self, shop_id)
  }
}
