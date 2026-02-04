use sqlx::PgPool;

use crate::config::Config;
use crate::services::{AuthService, GuestService, InviteService, SessionService, UserService};
use infra::services::{EmailService, EmailServiceConfig};

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub auth_service: AuthService,
  pub session_service: SessionService,
  pub invite_service: InviteService,
  pub user_service: UserService,
  pub guest_service: GuestService,
  pub pool: PgPool,
}

impl AppState {
  pub fn new(config: &Config, pool: PgPool) -> Self {
    let email_config = EmailServiceConfig {
      host: config.smtp_host.clone(),
      port: config.smtp_port,
      username: config.smtp_username.expose().to_string(),
      password: config.smtp_password.expose().to_string(),
      from: config.smtp_from.clone(),
    };

    let email_service = EmailService::new(email_config);
    let auth_service = AuthService::new(pool.clone());
    let user_service = UserService::new(pool.clone());
    let guest_service = GuestService::new(pool.clone());
    let invite_service = InviteService::new(pool.clone(), email_service, auth_service.clone());

    Self {
      config: config.clone(),
      auth_service,
      session_service: SessionService::new(pool.clone(), config.session_expiration_days),
      invite_service,
      user_service,
      guest_service,
      pool,
    }
  }
}
