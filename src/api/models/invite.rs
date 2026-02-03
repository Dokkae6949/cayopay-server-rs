use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::Role;

#[derive(Deserialize, Validate, ToSchema)]
pub struct InviteRequest {
  #[validate(email)]
  #[schema(example = "friend@example.com")]
  pub email: String,

  pub role: Role,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct AcceptInviteRequest {
  #[validate(length(min = 1, max = 127))]
  #[schema(example = "John")]
  pub first_name: String,
  #[validate(length(min = 1, max = 127))]
  #[schema(example = "Doe")]
  pub last_name: String,
  #[validate(length(min = 8, max = 127))]
  #[schema(example = "password123")]
  pub password: String,
}
