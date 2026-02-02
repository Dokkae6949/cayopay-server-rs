use crate::domain::Role;
use crate::state::AppState;
use crate::stores::UserStore;

pub async fn seed_database(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
  if UserStore::find_by_email(&state.pool, &state.config.owner_email)
    .await?
    .is_none()
  {
    tracing::info!("Seeding default owner user...");
    state
      .auth_service
      .register(
        state.config.owner_email.clone(),
        state.config.owner_password.clone(),
        state.config.owner_first_name.clone(),
        state.config.owner_last_name.clone(),
        Role::Owner,
      )
      .await?;
  }

  Ok(())
}
