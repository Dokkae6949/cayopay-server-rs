use sqlx::PgPool;

use crate::features::auth::service::AuthService;
use crate::features::guest_management::service::GuestManagementService;
use crate::features::onboarding::service::OnboardingService;
use crate::features::user_management::service::UserManagementService;
use crate::shared::config::Config;
use crate::shared::services::EmailService;

/// Application state containing all services and shared resources
#[derive(Clone)]
pub struct AppState {
  pub pool: PgPool,
  pub config: Config,
  pub auth_service: AuthService,
  pub onboarding_service: OnboardingService,
  pub user_management_service: UserManagementService,
  pub guest_management_service: GuestManagementService,
}

impl AppState {
  pub fn new(config: &Config, pool: PgPool) -> Self {
    let email_service = EmailService::new(config.email_config());
    let auth_service = AuthService::new(pool.clone());
    let onboarding_service =
      OnboardingService::new(pool.clone(), email_service, auth_service.clone());
    let user_management_service = UserManagementService::new(pool.clone());
    let guest_management_service = GuestManagementService::new(pool.clone());

    Self {
      pool,
      config: config.clone(),
      auth_service,
      onboarding_service,
      user_management_service,
      guest_management_service,
    }
  }
}
