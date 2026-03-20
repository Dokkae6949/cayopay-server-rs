use crate::{
  error::AppError,
  models::wallet::WalletLabel,
  services::auth,
  state::AppState,
  stores::{models::WalletCreation, WalletStore},
};

pub async fn owner(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
  match auth::register(
    &state.pool,
    state.config.owner_email.clone(),
    state.config.owner_password.clone(),
    state.config.owner_first_name.clone(),
    state.config.owner_last_name.clone(),
  )
  .await
  {
    Ok(_) => tracing::info!("Seeded default owner user"),
    Err(AppError::UserAlreadyExists) => {
      tracing::debug!("Default owner user already exists");
    }
    Err(e) => {
      tracing::warn!("Failed to seed owner user: {}", e);
      return Err(Box::new(e));
    }
  }
  Ok(())
}

pub async fn wallets(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
  let mut conn = state.pool.acquire().await?;

  for label in WalletLabel::variants() {
    match WalletStore::create(
      &mut *conn,
      &WalletCreation {
        owner: None,
        label: Some(label.clone()),
        allow_overdraft: true,
      },
    )
    .await
    {
      Ok(_) => tracing::info!("Seeded wallet with label {:?}", label),
      Err(sqlx::Error::Database(db_err))
        if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation =>
      {
        tracing::debug!("Wallet with label {:?} already exists", label);
      }
      Err(e) => {
        tracing::warn!("Failed to seed wallet with label {:?}: {}", label, e);
        return Err(Box::new(e));
      }
    }
  }

  Ok(())
}
