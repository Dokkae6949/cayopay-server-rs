use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
  #[validate(email)]
  #[schema(example = "user@example.com")]
  pub email: String,

  #[validate(length(min = 1))]
  #[schema(example = "password123")]
  pub password: String,
}
