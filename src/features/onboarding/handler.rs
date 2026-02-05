use axum::{
  extract::{Path, State},
  routing::{get, post},
  Json, Router,
};

use crate::shared::error::AppResult;
use crate::shared::extractors::{Authz, ValidatedJson};
use crate::shared::state::AppState;
use domain::{Email, Permission, RawPassword};

use super::models::{AcceptInviteRequest, InviteRequest, InviteResponse};

/// Send an invitation to join the platform
#[utoipa::path(
  post,
  path = "/api/invites",
  request_body = InviteRequest,
  responses(
    (status = StatusCode::OK, description = "Invite sent successfully"),
    (status = StatusCode::BAD_REQUEST, description = "Validation error"),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized"),
    (status = StatusCode::FORBIDDEN, description = "Forbidden"),
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn send_invite(
  State(state): State<AppState>,
  authz: Authz,
  ValidatedJson(payload): ValidatedJson<InviteRequest>,
) -> AppResult<()> {
  // Check permissions
  authz.require(Permission::SendInvite)?;
  authz.can_assign(payload.role)?;

  let email = Email::new(payload.email);
  let user = authz.0;

  state
    .onboarding_service
    .send_invite(user.id, email, payload.role)
    .await?;

  Ok(())
}

/// Get all invites (returns rich data with invitor names)
#[utoipa::path(
  get,
  path = "/api/invites",
  responses(
    (status = StatusCode::OK, description = "List of invites", body = [InviteResponse]),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized"),
    (status = StatusCode::FORBIDDEN, description = "Forbidden"),
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn get_invites(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<InviteResponse>>> {
  authz.require(Permission::ViewInvite)?;

  let invites = state.onboarding_service.get_all_invites().await?;

  Ok(Json(invites))
}

/// Accept an invite and register
#[utoipa::path(
  post,
  path = "/api/invites/{token}/accept",
  request_body = AcceptInviteRequest,
  params(
    ("token" = String, Path, description = "Invite token")
  ),
  responses(
    (status = StatusCode::OK, description = "Invite accepted successfully"),
    (status = StatusCode::BAD_REQUEST, description = "Validation error or expired invite"),
    (status = StatusCode::NOT_FOUND, description = "Invite not found"),
  ),
)]
pub async fn accept_invite(
  State(state): State<AppState>,
  Path(token): Path<String>,
  ValidatedJson(payload): ValidatedJson<AcceptInviteRequest>,
) -> AppResult<()> {
  state
    .onboarding_service
    .accept_invite(
      &token,
      RawPassword::new(payload.password),
      payload.first_name,
      payload.last_name,
    )
    .await?;

  Ok(())
}

/// Create router for onboarding/invite endpoints
pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", post(send_invite))
    .route("/", get(get_invites))
    .route("/:token/accept", post(accept_invite))
}
