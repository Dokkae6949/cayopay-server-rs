use domain::{Email, Role, User, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Request payload for user login
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
  #[validate(email(message = "Invalid email format"))]
  pub email: String,
  #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
  pub password: String,
}

/// Response containing user information
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
  pub id: UserId,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
}

impl From<User> for UserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      email: user.email,
      first_name: user.first_name,
      last_name: user.last_name,
      role: user.role,
    }
  }
}

/// Session information returned on login
#[derive(Debug, Serialize)]
pub struct SessionInfo {
  pub token: String,
  pub expires_in_ms: i64,
}
