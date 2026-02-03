use crate::domain::Role;
use crate::state::AppState;
use crate::stores::UserStore;
use crate::types::{Email, RawPassword};

pub async fn seed_database(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
  let owner_email = Email::new(&state.config.owner_email);

  if UserStore::find_by_email(&state.pool, &owner_email)
    .await?
    .is_none()
  {
    tracing::info!("Seeding default owner user...");
    state
      .auth_service
      .register(
        owner_email,
        RawPassword::new(&state.config.owner_password),
        state.config.owner_first_name.clone(),
        state.config.owner_last_name.clone(),
        Role::Owner,
      )
      .await?;
  }

  Ok(())
}
