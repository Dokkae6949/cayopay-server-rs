use sqlx::PgPool;

use crate::config::Config;
use crate::services::auth::AuthService;
use crate::services::session::SessionService;
use crate::services::{EmailService, InviteService};

#[derive(Clone)]
pub struct AppState {
  pub auth_service: AuthService,
  pub session_service: SessionService,
  pub invite_service: InviteService,
  pub pool: PgPool, // Kept for health check or other raw db needs
}

impl AppState {
  pub fn new(config: &Config, pool: PgPool) -> Self {
    let email_service = EmailService::new(config);
    let invite_service = InviteService::new(pool.clone(), email_service);

    Self {
      auth_service: AuthService::new(pool.clone()),
      session_service: SessionService::new(pool.clone()),
      invite_service,
      pool,
    }
  }
}
