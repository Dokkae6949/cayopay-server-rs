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
  // Tests disabled - they require a valid database connection
  // TODO: Add integration tests with test database setup
}
