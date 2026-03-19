use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use utoipa::ToSchema;

use crate::models::UserId;
use crate::services::email::EmailError;

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

  #[error("Permission denied: requires '{permission}' in scope '{scope}'")]
  PermissionDenied { permission: String, scope: String },

  #[error("User already exists")]
  UserAlreadyExists,

  #[error("Invite already sent")]
  InviteAlreadySent,

  #[error("Invite expired")]
  InviteExpired,

  #[error("Invitor with user id '{0}' does not exist")]
  InvitorMissing(UserId),

  #[error("Email error: {0}")]
  Email(#[from] EmailError),

  #[error("Validation error: {0}")]
  Validation(String),

  #[error("Bad request: {0}")]
  BadRequest(String),

  #[error("Internal server error")]
  InternalServerError,

  #[error("Password hashing error: {0}")]
  PasswordHash(#[from] argon2::password_hash::Error),
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ErrorResponse {
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<HashMap<String, Vec<String>>>,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, message) = match &self {
      AppError::Database(e) => {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
      }
      AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
      AppError::Authentication => (StatusCode::UNAUTHORIZED, "Authentication failed".to_string()),
      AppError::Authorization => (StatusCode::FORBIDDEN, "Permission denied".to_string()),
      AppError::PermissionDenied { permission, scope } => (
        StatusCode::FORBIDDEN,
        format!("Permission denied: requires '{permission}' in scope '{scope}'"),
      ),
      AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists".to_string()),
      AppError::InviteAlreadySent => (StatusCode::CONFLICT, "Invite already sent".to_string()),
      AppError::InvitorMissing(user_id) => {
        tracing::error!("Invitor missing: {:?}", user_id);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
      }
      AppError::InviteExpired => (StatusCode::BAD_REQUEST, "Invite expired".to_string()),
      AppError::Email(e) => {
        tracing::error!("Email error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
      }
      AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
      AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
      AppError::InternalServerError => {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
      }
      AppError::PasswordHash(e) => {
        tracing::error!("Password hash error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
      }
    };

    (status, Json(ErrorResponse { message, details: None })).into_response()
  }
}
