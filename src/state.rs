use sqlx::PgPool;

use crate::config::Config;
use crate::services::auth::AuthService;
use crate::services::session::SessionService;
use crate::services::{EmailService, InviteService};

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub auth_service: AuthService,
  pub session_service: SessionService,
  pub invite_service: InviteService,
  pub pool: PgPool,
}

impl AppState {
  pub fn new(config: &Config, pool: PgPool) -> Self {
    let auth_service = AuthService::new(pool.clone());
    let email_service = EmailService::new(config);
    let invite_service =
      InviteService::new(pool.clone(), email_service.clone(), auth_service.clone());

    Self {
      config: config.clone(),
      auth_service,
      session_service: SessionService::new(pool.clone(), config.session_expiration_days),
      invite_service,
      pool,
    }
  }
}
