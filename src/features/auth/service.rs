use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::shared::stores::{ActorStore, SessionStore, UserStore, WalletStore};
use domain::{Email, RawPassword, Role, Session, User};

/// Authentication and session management service
#[derive(Clone)]
pub struct AuthService {
  pool: PgPool,
}

impl AuthService {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Authenticate user with email and password, create a session
  pub async fn login(&self, email: Email, password: RawPassword) -> AppResult<(User, Session)> {
    // Find user by email
    let user = UserStore::find_by_email(&self.pool, &email)
      .await?
      .ok_or(AppError::Authentication)?;

    // Verify password
    if !user.password.verify(&password)? {
      return Err(AppError::Authentication);
    }

    // Create session
    let session = self.create_session(user.id).await?;

    Ok((user, session))
  }

  /// Get current user information from session token
  pub async fn me(&self, user: User) -> AppResult<User> {
    Ok(user)
  }

  /// Register a new user (used by onboarding flow)
  pub async fn register(
    &self,
    email: Email,
    password: RawPassword,
    first_name: String,
    last_name: String,
    role: Role,
  ) -> AppResult<User> {
    // Check if user already exists
    if UserStore::find_by_email(&self.pool, &email)
      .await?
      .is_some()
    {
      return Err(AppError::UserAlreadyExists);
    }

    let mut tx = self.pool.begin().await?;

    // Create actor (base entity)
    let actor = ActorStore::create(&mut *tx).await?;

    // Create user
    let user = UserStore::create(
      &mut *tx,
      email,
      password.hash()?,
      first_name,
      last_name,
      role,
      actor,
    )
    .await?;

    // Create default wallet for user
    WalletStore::create(&mut *tx, Some(actor), None, false).await?;

    tx.commit().await?;

    Ok(user)
  }

  /// Create a new session for a user
  async fn create_session(&self, user_id: domain::UserId) -> AppResult<Session> {
    let token = Uuid::new_v4().to_string();
    let expires_in = Duration::days(7);

    let session = SessionStore::create(&self.pool, user_id, token, None, None, expires_in).await?;

    Ok(session)
  }

  /// Validate session token and return user
  pub async fn validate_session(&self, token: &str) -> AppResult<User> {
    let session = SessionStore::find_by_token(&self.pool, token)
      .await?
      .ok_or(AppError::Unauthorized)?;

    if session.is_expired() {
      SessionStore::delete_by_id(&self.pool, &session.id).await?;
      return Err(AppError::Unauthorized);
    }

    let user = UserStore::find_by_id(&self.pool, &session.user_id)
      .await?
      .ok_or(AppError::Unauthorized)?;

    Ok(user)
  }
}
