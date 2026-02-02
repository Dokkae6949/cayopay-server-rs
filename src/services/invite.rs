use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  domain::{Actor, Invite, Role, User},
  error::{AppError, AppResult},
  services::EmailService,
  stores::{ActorStore, InviteStore, UserStore},
  types::{Email, Id, RawPassword},
};

#[derive(Clone)]
pub struct InviteService {
  pool: PgPool,
  email_service: EmailService,
}

impl InviteService {
  pub fn new(pool: PgPool, email_service: EmailService) -> Self {
    Self {
      pool,
      email_service,
    }
  }

  pub async fn create_invite(
    &self,
    created_by: Id<User>,
    email: Email,
    role: Role,
  ) -> AppResult<Invite> {
    if UserStore::find_by_email(&self.pool, &email)
      .await?
      .is_some()
    {
      return Err(AppError::UserAlreadyExists);
    }

    InviteStore::delete_expired_by_email(&self.pool, &email, Utc::now()).await?;

    let token = Uuid::new_v4().to_string();

    let invite = Invite {
      id: Id::new(),
      created_by,
      email: email.clone(),
      token: token.clone(),
      role,
      expires_at: Utc::now() + Duration::days(7),
      created_at: Utc::now(),
    };

    let invite = InviteStore::save(&self.pool, &invite).await?;
    let inviter_name = UserStore::find_by_id(&self.pool, &created_by)
      .await?
      .map(|u| format!("{} {}", u.first_name, u.last_name))
      .unwrap_or_else(|| "Someone".to_string());

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

    if invite.expires_at < Utc::now() {
      return Err(AppError::InviteExpired);
    }

    let user = User::new(
      invite.email.clone(),
      password.hash()?,
      first_name,
      last_name,
      invite.role,
    );
    let actor = Actor::new(user.actor_id);

    let mut tx = self.pool.begin().await?;

    ActorStore::save(&mut *tx, &actor).await?;
    UserStore::save(&mut *tx, user.clone()).await?;
    InviteStore::delete_by_id(&mut *tx, &invite.id).await?;

    tx.commit().await?;

    Ok(user)
  }
}
