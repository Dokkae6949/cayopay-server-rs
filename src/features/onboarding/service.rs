use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::features::auth::service::AuthService;
use crate::shared::error::{AppError, AppResult};
use crate::shared::services::EmailService;
use crate::shared::stores::{InviteStore, UserStore};
use domain::{Email, Invite, RawPassword, Role, User, UserId};

use super::models::InviteResponse;

/// Service for user onboarding via invitations
#[derive(Clone)]
pub struct OnboardingService {
  pool: PgPool,
  email_service: EmailService,
  auth_service: AuthService,
}

impl OnboardingService {
  pub fn new(pool: PgPool, email_service: EmailService, auth_service: AuthService) -> Self {
    Self {
      pool,
      email_service,
      auth_service,
    }
  }

  /// Send an invitation to a user
  pub async fn send_invite(
    &self,
    invitor_id: UserId,
    email: Email,
    role: Role,
  ) -> AppResult<Invite> {
    // Check if invite already exists for this email
    if let Some(invite) = InviteStore::find_by_email(&self.pool, &email).await? {
      if invite.is_expired() {
        // Delete expired invite
        InviteStore::delete_by_id(&self.pool, &invite.id).await?;
      } else {
        return Err(AppError::InviteAlreadySent);
      }
    }

    // Get inviter name for email
    let inviter = UserStore::find_by_id(&self.pool, &invitor_id)
      .await?
      .ok_or(AppError::InvitorMissing(invitor_id))?;
    let inviter_name = format!("{} {}", inviter.first_name, inviter.last_name);

    // Generate unique token
    let token = Uuid::new_v4().to_string();
    let expires_in = Duration::days(7);

    // Create invite
    let invite = InviteStore::create(
      &self.pool,
      invitor_id,
      email.clone(),
      token.clone(),
      role,
      expires_in,
    )
    .await?;

    // Send email
    self
      .email_service
      .send_invite(&email, &token, &inviter_name)
      .await?;

    Ok(invite)
  }

  /// Accept an invite and register a new user
  pub async fn accept_invite(
    &self,
    token: &str,
    password: RawPassword,
    first_name: String,
    last_name: String,
  ) -> AppResult<User> {
    // Find invite by token
    let invite = InviteStore::find_by_token(&self.pool, token)
      .await?
      .ok_or(AppError::NotFound)?;

    // Check if expired
    if invite.is_expired() {
      return Err(AppError::InviteExpired);
    }

    // Register user
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

    // Delete invite after successful registration
    InviteStore::delete_by_id(&self.pool, &invite.id).await?;

    Ok(user)
  }

  /// Get all invites (returns rich data with invitor names)
  pub async fn get_all_invites(&self) -> AppResult<Vec<InviteResponse>> {
    let invites = InviteStore::list_all(&self.pool).await?;

    let mut responses = Vec::new();
    for invite in invites {
      // Get invitor name for each invite
      let inviter = UserStore::find_by_id(&self.pool, &invite.invitor)
        .await?
        .ok_or(AppError::InvitorMissing(invite.invitor))?;
      let inviter_name = format!("{} {}", inviter.first_name, inviter.last_name);

      responses.push(InviteResponse {
        id: invite.id,
        invitor_id: invite.invitor,
        invitor_name,
        email: invite.email,
        token: invite.token,
        role: invite.role,
        status: invite.status,
        expires_at: invite.created_at + invite.expires_in,
        created_at: invite.created_at,
      });
    }

    Ok(responses)
  }
}
