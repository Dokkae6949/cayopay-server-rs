use crate::{
  api::{
    extractor::{Authz, ValidatedJson},
    models::{AcceptInviteRequest, InviteRequest},
  },
  domain::{Permission, Role},
  error::{AppResult, ErrorResponse},
  state::AppState,
  types::Email,
};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

#[utoipa::path(
  post,
  context_path = "/api/invites",
  path = "/",
  request_body = InviteRequest,
  responses(
    (status = StatusCode::OK, description = "Invite sent successfully"),
    (status = StatusCode::BAD_REQUEST, description = "Validation error", body = ErrorResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
    (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
    (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn create_invite(
  State(state): State<AppState>,
  authz: Authz,
  ValidatedJson(payload): ValidatedJson<InviteRequest>,
) -> AppResult<()> {
  authz.require(Permission::InviteUser)?;
  authz.can_assign(payload.role)?;

  let email = Email::new(payload.email);
  let user = authz.0;

  state
    .invite_service
    .create_invite(user.id, email, payload.role)
    .await?;

  Ok(())
}

#[utoipa::path(
  post,
  context_path = "/api/invites",
  path = "/{token}/accept",
  request_body = AcceptInviteRequest,
  params(
    ("token" = String, Path, description = "Invite token")
  ),
  responses(
    (status = StatusCode::OK, description = "Invite accepted successfully"),
    (status = StatusCode::BAD_REQUEST, description = "Validation error or expired invite", body = ErrorResponse),
    (status = StatusCode::NOT_FOUND, description = "Invite not found", body = ErrorResponse),
    (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse)
  ),
)]
pub async fn accept_invite(
  State(state): State<AppState>,
  axum::extract::Path(token): axum::extract::Path<String>,
  ValidatedJson(payload): ValidatedJson<AcceptInviteRequest>,
) -> AppResult<()> {
  state
    .invite_service
    .accept_invite(
      &token,
      crate::types::RawPassword::new(payload.password),
      payload.first_name,
      payload.last_name,
    )
    .await?;

  Ok(())
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_invite))
    .route("/:token/accept", post(accept_invite))
}
