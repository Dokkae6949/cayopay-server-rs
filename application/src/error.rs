use domain::UserId;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("Entity not found")]
  NotFound,

  #[error("Authentication failed")]
  Authentication,

  #[error("Authorization failed")]
  Authorization,

  #[error("User already exists")]
  UserAlreadyExists,

  #[error("Invite already sent")]
  InviteAlreadySent,

  #[error("Invite expired")]
  InviteExpired,

  #[error("Invitor with user id '{0}' does not exist")]
  InvitorMissing(UserId),

  #[error("Email error: {0}")]
  Email(#[from] infra::services::EmailError),

  #[error("Validation error: {0}")]
  Validation(String),

  #[error("Bad request: {0}")]
  BadRequest(String),

  #[error("Internal server error")]
  InternalServerError,

  #[error("Password hashing error: {0}")]
  PasswordHash(#[from] argon2::password_hash::Error),
}
