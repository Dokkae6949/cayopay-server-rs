use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use domain::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use utoipa::ToSchema;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("Entity not found")]
  NotFound,

  #[error("Authentication failed")]
  Authentication,

  #[error("Unauthorized")]
  Unauthorized,

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
  Email(String),

  #[error("Validation error: {0}")]
  Validation(String),

  #[error("Bad request: {0}")]
  BadRequest(String),

  #[error("Internal server error")]
  InternalServerError,

  #[error("Password hashing error: {0}")]
  PasswordHash(#[from] argon2::password_hash::Error),
}

// Email service error wrapper
impl From<lettre::transport::smtp::Error> for AppError {
  fn from(e: lettre::transport::smtp::Error) -> Self {
    AppError::Email(e.to_string())
  }
}

impl From<lettre::error::Error> for AppError {
  fn from(e: lettre::error::Error) -> Self {
    AppError::Email(e.to_string())
  }
}

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
  fn from(inner: AppError) -> Self {
    ApiError(inner)
  }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ErrorResponse {
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<HashMap<String, Vec<String>>>,
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (status, message, details) = match self.0 {
      AppError::Database(e) => {
        tracing::error!("Database error: {:?}", e);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Internal server error".to_string(),
          None,
        )
      }
      AppError::NotFound => (
        StatusCode::NOT_FOUND,
        "Resource not found".to_string(),
        None,
      ),
      AppError::Authentication => (
        StatusCode::UNAUTHORIZED,
        "Authentication failed".to_string(),
        None,
      ),
      AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string(), None),
      AppError::Authorization => (StatusCode::FORBIDDEN, "Permission denied".to_string(), None),
      AppError::UserAlreadyExists => (
        StatusCode::CONFLICT,
        "User already exists".to_string(),
        None,
      ),
      AppError::InviteAlreadySent => (
        StatusCode::CONFLICT,
        "Invite already sent".to_string(),
        None,
      ),
      AppError::InvitorMissing(user_id) => {
        tracing::error!("Invitor missing: {:?}", user_id);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Internal server error".to_string(),
          None,
        )
      }
      AppError::InviteExpired => (StatusCode::BAD_REQUEST, "Invite expired".to_string(), None),
      AppError::Email(e) => {
        tracing::error!("Email error: {}", e);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Internal server error".to_string(),
          None,
        )
      }
      AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg, None),
      AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, None),
      AppError::InternalServerError => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
        None,
      ),
      AppError::PasswordHash(e) => {
        tracing::error!("Password hash error: {:?}", e);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Internal server error".to_string(),
          None,
        )
      }
    };

    let body = Json(ErrorResponse { message, details });

    (status, body).into_response()
  }
}
