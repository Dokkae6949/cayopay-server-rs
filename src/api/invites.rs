use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
  api::extractor::AuthUser,
  app_state::AppState,
  error::AppResult,
  types::Email,
};

#[derive(Deserialize, Validate, ToSchema)]
pub struct InviteRequest {
  #[validate(email)]
  #[schema(example = "friend@example.com")]
  pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct InviteResponse {
  pub message: String,
}

#[utoipa::path(
    post,
    context_path = "/api/invites",
    path = "/",
    request_body = InviteRequest,
    responses(
        (status = StatusCode::OK, description = "Invite sent successfully", body = InviteResponse),
        (status = StatusCode::BAD_REQUEST, description = "Validation error", body = ErrorResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_invite(
  State(state): State<AppState>,
  AuthUser(user): AuthUser,
  Json(payload): Json<InviteRequest>,
) -> AppResult<Json<InviteResponse>> {
  payload.validate()?;

  let email = Email::new(payload.email);

  state.invite_service.create_invite(user.id, email).await?;

  Ok(Json(InviteResponse {
    message: "Invite sent successfully".to_string(),
  }))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", post(create_invite))
}
