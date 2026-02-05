use axum::{
  async_trait,
  extract::{FromRequest, FromRequestParts, Request},
  http::request::Parts,
  RequestPartsExt,
};
use axum_extra::extract::CookieJar;
use std::ops::Deref;

use crate::shared::error::{ApiError, AppError};
use crate::shared::state::AppState;
use domain::User;

pub struct Authn(pub User);

impl Deref for Authn {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub struct AuthnWithState {
  pub user: User,
  pub state: AppState,
}

#[async_trait]
impl FromRequestParts<AppState> for Authn {
  type Rejection = ApiError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let jar = parts
      .extract::<CookieJar>()
      .await
      .map_err(|_| AppError::Authentication)?;

    let session_cookie = jar
      .get(&state.config.session_cookie_name)
      .ok_or(AppError::Authentication)?;
    let token = session_cookie.value();

    let user = state.auth_service.validate_session(token).await?;

    Ok(Authn(user))
  }
}

#[async_trait]
impl FromRequest<AppState> for Authn {
  type Rejection = ApiError;

  async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
    let (mut parts, _) = req.into_parts();
    Self::from_request_parts(&mut parts, state).await
  }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthnWithState {
  type Rejection = ApiError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let user = Authn::from_request_parts(parts, state).await?.0;
    Ok(AuthnWithState {
      user,
      state: state.clone(),
    })
  }
}

#[async_trait]
impl FromRequest<AppState> for AuthnWithState {
  type Rejection = ApiError;

  async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
    let (mut parts, _) = req.into_parts();
    Self::from_request_parts(&mut parts, state).await
  }
}
