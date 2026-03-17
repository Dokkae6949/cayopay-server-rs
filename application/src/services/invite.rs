use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  error::{AppError, AppResult},
  services::{auth::AuthService, AuthorizationService},
};
use domain::{models::permission::Permission, Email, Invite, RawPassword, User, UserId};
use infra::{
  services::EmailService,
  stores::{
    models::{InviteCreation, UserRoleCreation},
    InviteStore, RoleStore, UserRoleStore, UserStore,
  },
};

#[derive(Clone)]
pub struct InviteService {
  pool: PgPool,
  email_service: EmailService,
  auth_service: AuthService,
  authz_service: AuthorizationService,
}

impl InviteService {
  pub fn new(
    pool: PgPool,
    email_service: EmailService,
    auth_service: AuthService,
    authz_service: AuthorizationService,
  ) -> Self {
    Self {
      pool,
      email_service,
      auth_service,
      authz_service,
    }
  }

  pub async fn create_invite(
    &self,
    invitor: UserId,
    email: Email,
    role: String,
  ) -> AppResult<Invite> {
    self.authz_service.require(invitor, Permission::SendInvite).await?;

    // Ensure the role exists before sending an invite
    if RoleStore::find_by_name(&self.pool, &role).await?.is_none() {
      return Err(AppError::BadRequest(format!(
        "Role '{}' does not exist",
        role
      )));
    }

    if let Some(invite) = InviteStore::find_by_email(&self.pool, &email).await? {
      if invite.is_expired() {
        InviteStore::delete_by_id(&self.pool, &invite.id).await?;
      } else {
        return Err(AppError::InviteAlreadySent);
      }
    }

    let inviter_name = UserStore::find_by_id(&self.pool, &invitor)
      .await?
      .map(|u| format!("{} {}", u.first_name, u.last_name))
      .ok_or(AppError::InvitorMissing(invitor))?;

    let token = Uuid::new_v4().to_string();

    let new_invite = InviteCreation {
      invitor,
      email: email.clone(),
      token: token.clone(),
      role,
      expires_in: Duration::days(7),
    };

    let invite = InviteStore::create(&self.pool, &new_invite).await?;

    self
      .email_service
      .send_invite(&email, &token, &inviter_name)
      .await?;

    Ok(invite)
  }

  pub async fn accept_invite(
    &self,
    token: &str,
    password: RawPassword,
    first_name: String,
    last_name: String,
  ) -> AppResult<User> {
    let invite = InviteStore::find_by_token(&self.pool, token)
      .await?
      .ok_or(AppError::NotFound)?;

    if invite.is_expired() {
      return Err(AppError::InviteExpired);
    }

    let user = self
      .auth_service
      .register(invite.email.clone(), password, first_name, last_name)
      .await?;

    // Assign the role from the invite to the new user (role may no longer exist)
    if let Some(role) = RoleStore::find_by_name(&self.pool, &invite.role).await? {
      UserRoleStore::assign(
        &self.pool,
        &UserRoleCreation {
          user_id: user.id,
          role_id: role.id,
        },
      )
      .await?;
    } else {
      // The role was deleted after the invite was created. Delete the user and
      // surface an error so the invite can be re-issued with a valid role.
      return Err(AppError::BadRequest(format!(
        "The role '{}' no longer exists; the invite must be re-issued",
        invite.role
      )));
    }

    InviteStore::delete_by_id(&self.pool, &invite.id).await?;

    Ok(user)
  }

  pub async fn get_all(&self, caller: UserId) -> AppResult<Vec<Invite>> {
    self.authz_service.require(caller, Permission::ViewInvite).await?;
    Ok(InviteStore::list_all(&self.pool).await?)
  }
}
