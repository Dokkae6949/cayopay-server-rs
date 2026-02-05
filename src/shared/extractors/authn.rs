use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;
use std::ops::Deref;

use crate::error::{ApiError, AppError};
use crate::state::AppState;
use domain::User;

pub struct Authn(pub User);

impl Deref for Authn {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
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
