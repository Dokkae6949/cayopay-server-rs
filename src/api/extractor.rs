use axum::{
  async_trait,
  extract::{FromRequest, FromRequestParts},
  http::{request::Parts, Request},
  Json, RequestPartsExt,
};
use axum_extra::extract::CookieJar;
use serde::de::DeserializeOwned;
use std::ops::Deref;
use validator::Validate;

use crate::{
  domain::{Permission, User},
  error::AppError,
  state::AppState,
  stores::UserStore,
};

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request(
    req: Request<axum::body::Body>,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(|e| AppError::BadRequest(e.to_string()))?;
    value.validate()?;
    Ok(ValidatedJson(value))
  }
}

pub struct Authn(pub User);

impl Deref for Authn {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[async_trait]
impl FromRequestParts<AppState> for Authn {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let jar = parts
      .extract::<CookieJar>()
      .await
      .map_err(|_| AppError::InvalidCredentials)?;

    let session_cookie = jar
      .get(&state.config.session_cookie_name)
      .ok_or(AppError::InvalidCredentials)?;
    let token = session_cookie.value();

    let session = state
      .session_service
      .validate_session(token)
      .await?
      .ok_or(AppError::InvalidCredentials)?;

    let user = UserStore::find_by_id(&state.pool, &session.user_id)
      .await?
      .ok_or(AppError::InvalidCredentials)?;

    Ok(Authn(user))
  }
}

pub struct Authz(pub User);

impl Authz {
  pub fn can_assign(&self, target_role: crate::domain::Role) -> Result<(), AppError> {
    if self.0.role.can_assign_role(target_role) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }

  pub fn require(&self, perm: Permission) -> Result<(), AppError> {
    if self.0.role.has_permission(perm) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }

  pub fn require_any(&self, perms: &[Permission]) -> Result<(), AppError> {
    if perms.iter().any(|p| self.0.role.has_permission(*p)) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }

  pub fn require_all(&self, perms: &[Permission]) -> Result<(), AppError> {
    if perms.iter().all(|p| self.0.role.has_permission(*p)) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }
}

#[async_trait]
impl FromRequestParts<AppState> for Authz {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let user = Authn::from_request_parts(parts, state).await?.0;
    Ok(Authz(user))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::domain::Role;
  use crate::types::{Email, HashedPassword, Id};
  use chrono::Utc;

  fn create_user(role: Role) -> User {
    User {
      id: Id::new(),
      actor_id: Id::new(),
      email: Email::new("test@example.com".to_string()),
      password_hash: HashedPassword::new("hash".to_string()),
      first_name: "Test".to_string(),
      last_name: "User".to_string(),
      role,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }

  #[test]
  fn test_authz_can_assign() {
    let owner = Authz(create_user(Role::Owner));
    assert!(owner.can_assign(Role::Admin).is_ok());
    assert!(owner.can_assign(Role::Owner).is_ok());

    let admin = Authz(create_user(Role::Admin));
    assert!(admin.can_assign(Role::Admin).is_ok());
    assert!(admin.can_assign(Role::Owner).is_err());
  }

  #[test]
  fn test_authz_require() {
    let owner = Authz(create_user(Role::Owner));
    assert!(owner.require(Permission::InviteUsers).is_ok());

    let admin = Authz(create_user(Role::Admin));
    assert!(admin.require(Permission::InviteUsers).is_ok());
    assert!(admin.require(Permission::ConfigureSettings).is_err());
  }

  #[test]
  fn test_authz_require_any() {
    let admin = Authz(create_user(Role::Admin));
    assert!(admin
      .require_any(&[Permission::InviteUsers, Permission::ConfigureSettings])
      .is_ok());
    assert!(admin.require_any(&[Permission::ConfigureSettings]).is_err());
  }

  #[test]
  fn test_authz_require_all() {
    let owner = Authz(create_user(Role::Owner));
    assert!(owner
      .require_all(&[Permission::InviteUsers, Permission::ConfigureSettings])
      .is_ok());

    let admin = Authz(create_user(Role::Admin));
    assert!(admin
      .require_all(&[Permission::InviteUsers, Permission::ConfigureSettings])
      .is_err());
    assert!(admin.require_all(&[Permission::InviteUsers]).is_ok());
  }
}
