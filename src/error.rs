use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum AppError {
  #[error("Database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("Validation error: {0}")]
  Validation(#[from] validator::ValidationErrors),

  #[error("Missing or invalid credentials")]
  Authentication,

  #[error("Missing or insufficient permissions")]
  Authorization,

  #[error("Resource not found")]
  NotFound,

  #[error("User with this email already exists")]
  UserAlreadyExists,

  #[error("Invitation to the provided email has already been sent")]
  InviteAlreadySent,

  #[error("Invitation has expired")]
  InviteExpired,

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
    let (status, message, details) = match self {
      AppError::Database(e) => {
        tracing::error!("Database error: {:?}", e);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Internal server error".to_string(),
          None,
        )
      }
      AppError::Validation(e) => {
        let mut details = HashMap::new();
        for (field, errors) in e.field_errors().iter() {
          let messages: Vec<String> = errors
            .iter()
            .map(|err| {
              if let Some(msg) = &err.message {
                msg.to_string()
              } else {
                format!("Invalid value for code: {}", err.code)
              }
            })
            .collect();
          details.insert(field.to_string(), messages);
        }
        (
          StatusCode::BAD_REQUEST,
          "Validation error".to_string(),
          Some(details),
        )
      }
      AppError::Authentication => (StatusCode::UNAUTHORIZED, self.to_string(), None),
      AppError::Authorization => (StatusCode::FORBIDDEN, self.to_string(), None),
      AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string(), None),
      AppError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string(), None),
      AppError::InviteAlreadySent => (StatusCode::CONFLICT, self.to_string(), None),
      AppError::InviteExpired => (StatusCode::BAD_REQUEST, self.to_string(), None),
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

pub type AppResult<T> = Result<T, AppError>;
