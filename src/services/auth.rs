use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use crate::models::User;
use crate::stores::{
  models::{UserCreation, WalletCreation},
  ActorStore, UserStore, WalletStore,
};
use crate::types::{Email, RawPassword};

pub async fn register(
  pool: &PgPool,
  email: Email,
  password: RawPassword,
  first_name: String,
  last_name: String,
) -> AppResult<User> {
  let mut conn = pool.acquire().await?;

  if UserStore::find_by_email(&mut *conn, &email)
    .await?
    .is_some()
  {
    return Err(AppError::UserAlreadyExists);
  }

  let actor = ActorStore::create(&mut *conn).await?;

  let user = UserStore::create(
    &mut *conn,
    &UserCreation {
      actor_id: actor,
      email,
      password: password.hash()?,
      first_name,
      last_name,
    },
  )
  .await?;

  WalletStore::create(
    &mut *conn,
    &WalletCreation {
      owner: Some(actor),
      label: None,
      allow_overdraft: false,
    },
  )
  .await?;

  Ok(user)
}
