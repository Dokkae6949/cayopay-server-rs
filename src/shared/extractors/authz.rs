use crate::shared::error::{ApiError, AppError};
use crate::shared::extractors::Authn;
use crate::shared::state::AppState;
use axum::{
  async_trait,
  extract::{FromRequest, FromRequestParts, Request},
  http::request::Parts,
};
use domain::{Permission, Role, User};

pub struct Authz {
  pub user: User,
  pub state: AppState,
}

impl Authz {
  pub fn can_assign(&self, target_role: Role) -> Result<(), AppError> {
    if self.user.role.can_assign_role(target_role) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }

  pub fn require(&self, perm: Permission) -> Result<(), AppError> {
    if self.user.role.has_permission(perm) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }

  pub fn require_any(&self, perms: &[Permission]) -> Result<(), AppError> {
    if perms.iter().any(|p| self.user.role.has_permission(*p)) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
  }

  pub fn require_all(&self, perms: &[Permission]) -> Result<(), AppError> {
    if perms.iter().all(|p| self.user.role.has_permission(*p)) {
      Ok(())
    } else {
      Err(AppError::Authorization)
    }
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
      state: state.clone(),
    })
  }
}

#[async_trait]
impl FromRequest<AppState> for Authz {
  type Rejection = ApiError;

  async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
    let (mut parts, _) = req.into_parts();
    Self::from_request_parts(&mut parts, state).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use domain::{Email, HashedPassword, Id};

  fn create_user(role: Role) -> User {
    User {
      id: Id::new(),
      actor_id: Id::new(),
      email: Email::new("test@example.com".to_string()),
      password: HashedPassword::new("hash".to_string()),
      first_name: "Test".to_string(),
      last_name: "User".to_string(),
      role,
      created_at: Utc::now(),
      updated_at: None,
    }
  }

  fn create_authz(role: Role) -> Authz {
    use crate::shared::config::Config;
    use sqlx::PgPool;

    let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
    let config = Config::default();
    let state = AppState::new(&config, pool);

    Authz {
      user: create_user(role),
      state,
    }
  }

  #[test]
  fn test_authz_can_assign() {
    let owner = create_authz(Role::Owner);
    assert!(owner.can_assign(Role::Admin).is_ok());
    assert!(owner.can_assign(Role::Owner).is_ok());

    let admin = create_authz(Role::Admin);
    assert!(admin.can_assign(Role::Admin).is_ok());
    assert!(admin.can_assign(Role::Owner).is_err());
  }

  #[test]
  fn test_authz_require() {
    let owner = create_authz(Role::Owner);
    assert!(owner.require(Permission::SendInvite).is_ok());

    let admin = create_authz(Role::Admin);
    assert!(admin.require(Permission::SendInvite).is_ok());
    assert!(admin.require(Permission::ConfigureSettings).is_err());
  }

  #[test]
  fn test_authz_require_any() {
    let admin = create_authz(Role::Admin);
    assert!(admin
      .require_any(&[Permission::SendInvite, Permission::ConfigureSettings])
      .is_ok());
    assert!(admin.require_any(&[Permission::ConfigureSettings]).is_err());
  }

  #[test]
  fn test_authz_require_all() {
    let owner = create_authz(Role::Owner);
    assert!(owner
      .require_all(&[Permission::SendInvite, Permission::ConfigureSettings])
      .is_ok());

    let admin = create_authz(Role::Admin);
    assert!(admin
      .require_all(&[Permission::SendInvite, Permission::ConfigureSettings])
      .is_err());
    assert!(admin.require_all(&[Permission::SendInvite]).is_ok());
  }
}
