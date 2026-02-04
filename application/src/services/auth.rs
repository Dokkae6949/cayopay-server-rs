use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use domain::{Email, RawPassword, Role, User};
use infra::stores::{
  models::{UserCreation, WalletCreation},
  ActorStore, UserStore, WalletStore,
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
      .ok_or(AppError::Authentication)?;

    if !user.password.verify(&password)? {
      return Err(AppError::Authentication);
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
    if UserStore::find_by_email(&self.pool, &email)
      .await?
      .is_some()
    {
      return Err(AppError::UserAlreadyExists);
    }

    let mut tx = self.pool.begin().await?;

    let actor = ActorStore::create(&mut *tx).await?;

    let user = UserStore::create(
      &mut *tx,
      &UserCreation {
        actor_id: actor,
        email,
        password: password.hash()?,
        first_name,
        last_name,
        role,
      },
    )
    .await?;

    WalletStore::create(
      &mut *tx,
      &WalletCreation {
        owner: Some(actor),
        label: None,
        allow_overdraft: false,
      },
    )
    .await?;

    tx.commit().await?;

    Ok(user)
  }
}
