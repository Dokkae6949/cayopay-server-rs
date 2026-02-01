use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;
use std::ops::Deref;

use crate::{app_state::AppState, domain::User, error::AppError, stores::UserStore};

pub struct AuthUser(pub User);

impl Deref for AuthUser {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let jar = parts
      .extract::<CookieJar>()
      .await
      .map_err(|_| AppError::AuthError)?;

    let session_cookie = jar
      .get(&state.config.session_cookie_name)
      .ok_or(AppError::AuthError)?;
    let token = session_cookie.value();

    let session = state
      .session_service
      .validate_session(token)
      .await?
      .ok_or(AppError::AuthError)?;

    let user = UserStore::find_by_id(&state.pool, &session.user_id)
      .await?
      .ok_or(AppError::AuthError)?;

    Ok(AuthUser(user))
  }
}
