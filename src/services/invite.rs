use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
  domain::{Invite, User},
  error::AppResult,
  services::EmailService,
  stores::{InviteStore, UserStore},
  types::{Email, Id},
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

  pub async fn create_invite(&self, created_by: Id<User>, email: Email) -> AppResult<Invite> {
    let token = Uuid::new_v4().to_string();

    let invite = Invite {
      id: Id::new(),
      created_by,
      email: email.clone(),
      token: token.clone(),
      expires_at: Utc::now() + Duration::days(7),
      created_at: Utc::now(),
    };

    let invite = InviteStore::create(&self.pool, &invite).await?;
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
}
