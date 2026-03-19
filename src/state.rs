use sqlx::PgPool;

use crate::config::Config;
use crate::services::{AuthorizationService, EmailService, EmailServiceConfig};

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub pool: PgPool,
  pub authz_service: AuthorizationService,
  pub email_service: EmailService,
}

impl AppState {
  pub fn new(config: &Config, pool: PgPool) -> Self {
    let email_service = EmailService::new(EmailServiceConfig {
      host: config.smtp_host.clone(),
      port: config.smtp_port,
      username: config.smtp_username.expose().to_string(),
      password: config.smtp_password.expose().to_string(),
      from: config.smtp_from.clone(),
    });

    let authz_service = AuthorizationService::new(pool.clone());

    Self {
      config: config.clone(),
      pool,
      authz_service,
      email_service,
    }
  }
}
