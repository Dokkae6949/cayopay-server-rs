use sqlx::PgPool;

use crate::config::Config;
use crate::error::AppResult;
use crate::services::auth::AuthService;
use crate::services::session::SessionService;
use crate::services::{ActorService, EmailService, GuestService, InviteService, UserService};

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub auth_service: AuthService,
  pub actor_service: ActorService,
  pub user_service: UserService,
  pub guest_service: GuestService,
  pub session_service: SessionService,
  pub invite_service: InviteService,
  pub pool: PgPool,
}

impl AppState {
  pub fn new(config: &Config, pool: PgPool) -> AppResult<Self> {
    let auth_service = AuthService::new(pool.clone());
    let actor_service = ActorService::new(pool.clone());
    let user_service = UserService::new(pool.clone());
    let guest_service = GuestService::new(pool.clone());
    let email_service = EmailService::new(config)?;
    let invite_service =
      InviteService::new(pool.clone(), email_service.clone(), auth_service.clone());

    Ok(Self {
      config: config.clone(),
      auth_service,
      actor_service,
      user_service,
      guest_service,
      session_service: SessionService::new(pool.clone(), config.session_expiration_days),
      invite_service,
      pool,
    })
  }
}
