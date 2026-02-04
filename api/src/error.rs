use application::error::AppError;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
  fn from(inner: AppError) -> Self {
    ApiError(inner)
  }
}

pub type AppResult<T> = Result<T, ApiError>;

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
        tracing::error!("Email error: {:?}", e);
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
