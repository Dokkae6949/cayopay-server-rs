use axum::{
  extract::Path,
  http::StatusCode,
  routing::{get, post},
  Json, Router,
};

use crate::shared::error::AppResult;
use crate::shared::extractors::{Authz, ValidatedJson};
use crate::shared::state::AppState;
use domain::{Email, Permission, RawPassword};

use super::models::{AcceptInviteRequest, InviteRequest, InviteResponse};

/// Helper extractor that provides state without consuming request body
struct StateOnly(AppState);

#[axum::async_trait]
impl axum::extract::FromRequestParts<AppState> for StateOnly {
  type Rejection = std::convert::Infallible;

  async fn from_request_parts(
    _parts: &mut axum::http::request::Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    Ok(StateOnly(state.clone()))
  }
}

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
#[axum::debug_handler]
pub async fn send_invite(
  authz: Authz,
  ValidatedJson(payload): ValidatedJson<InviteRequest>,
) -> AppResult<()> {
  // Check permissions
  authz.require(Permission::SendInvite)?;
  authz.can_assign(payload.role)?;

  let email = Email::new(payload.email);
  let user_id = authz.user.id;

  authz
    .state
    .onboarding_service
    .send_invite(user_id, email, payload.role)
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
#[axum::debug_handler]
pub async fn get_invites(authz: Authz) -> AppResult<Json<Vec<InviteResponse>>> {
  authz.require(Permission::ViewInvite)?;

  let invites = authz.state.onboarding_service.get_all_invites().await?;

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
#[axum::debug_handler]
pub async fn accept_invite(
  StateOnly(state): StateOnly,
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
  Router::<AppState>::new()
    .route("/", post(send_invite))
    .route("/", get(get_invites))
    .route("/:token/accept", post(accept_invite))
}
