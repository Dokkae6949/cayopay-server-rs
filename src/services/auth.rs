use sqlx::PgPool;

use crate::{
  domain::{Actor, Role, User, Wallet},
  error::{AppError, AppResult},
  stores::{ActorStore, UserStore, WalletStore},
  types::{Email, RawPassword},
};

#[derive(Clone)]
pub struct AuthService {
  pool: PgPool,
}

impl AuthService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn login(&self, email: Email, password: RawPassword) -> AppResult<User> {
    let user = UserStore::find_by_email(&self.pool, &email)
      .await?
      .ok_or(AppError::InvalidCredentials)?;

    if !user.password_hash.verify(&password)? {
      return Err(AppError::InvalidCredentials);
    }

    Ok(user)
  }

  pub async fn register(
    &self,
    email: Email,
    password: RawPassword,
    first_name: String,
    last_name: String,
    role: Role,
  ) -> AppResult<User> {
    let actor = Actor::new();
    let user = User::new(
      actor.id,
      email,
      password.hash()?,
      first_name,
      last_name,
      role,
    );
    let wallet = Wallet::new(actor.id);

    let mut tx = self.pool.begin().await?;

    ActorStore::save(&mut *tx, &actor).await?;
    UserStore::save(&mut *tx, &user).await?;
    WalletStore::save(&mut *tx, &wallet).await?;

    tx.commit().await?;

    Ok(user)
  }
}
