use crate::{
  api::extractor::Authn,
  domain::{Permission, Role, User},
  error::AppError,
  state::AppState,
};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

pub struct Authz(pub User);

impl Authz {
  pub fn can_assign(&self, target_role: Role) -> Result<(), AppError> {
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
    assert!(owner.require(Permission::InviteUser).is_ok());

    let admin = Authz(create_user(Role::Admin));
    assert!(admin.require(Permission::InviteUser).is_ok());
    assert!(admin.require(Permission::ConfigureSettings).is_err());
  }

  #[test]
  fn test_authz_require_any() {
    let admin = Authz(create_user(Role::Admin));
    assert!(admin
      .require_any(&[Permission::InviteUser, Permission::ConfigureSettings])
      .is_ok());
    assert!(admin.require_any(&[Permission::ConfigureSettings]).is_err());
  }

  #[test]
  fn test_authz_require_all() {
    let owner = Authz(create_user(Role::Owner));
    assert!(owner
      .require_all(&[Permission::InviteUser, Permission::ConfigureSettings])
      .is_ok());

    let admin = Authz(create_user(Role::Admin));
    assert!(admin
      .require_all(&[Permission::InviteUser, Permission::ConfigureSettings])
      .is_err());
    assert!(admin.require_all(&[Permission::InviteUser]).is_ok());
  }
}
