use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  error::{AppError, AppResult},
  services::auth::AuthService,
};
use domain::{Email, Invite, RawPassword, Role, User, UserId};
use infra::{
  services::EmailService,
  stores::{models::InviteCreation, InviteStore, UserStore},
};

#[derive(Clone)]
pub struct InviteService {
  pool: PgPool,
  email_service: EmailService,
  auth_service: AuthService,
}

impl InviteService {
  pub fn new(pool: PgPool, email_service: EmailService, auth_service: AuthService) -> Self {
    Self {
      pool,
      email_service,
      auth_service,
    }
  }

  pub async fn create_invite(
    &self,
    invitor: UserId,
    email: Email,
    role: Role,
  ) -> AppResult<Invite> {
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
      .register(
        invite.email.clone(),
        password,
        first_name,
        last_name,
        invite.role,
      )
      .await?;

    InviteStore::delete_by_id(&self.pool, &invite.id).await?;

    Ok(user)
  }

  pub async fn get_all(&self) -> AppResult<Vec<Invite>> {
    Ok(InviteStore::get_all(&self.pool).await?)
  }
}
