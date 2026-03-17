use crate::{
  error::AppResult,
  extractor::{Authn, ValidatedJson},
  models::{AcceptInviteRequest, InviteRequest, InviteResponse},
};
use application::{services::invite, state::AppState};
use axum::{
  extract::{Path, State},
  routing::{get, post},
  Json, Router,
};
use domain::{Email, RawPassword};

#[utoipa::path(
  post,
  path = "/api/invites",
  request_body = InviteRequest,
  responses(
    (status = StatusCode::OK, description = "Invite sent successfully"),
    (status = StatusCode::BAD_REQUEST, description = "Validation error", body = ErrorResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
    (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn create_invite(
  State(state): State<AppState>,
  authn: Authn,
  ValidatedJson(payload): ValidatedJson<InviteRequest>,
) -> AppResult<()> {
  let email = Email::new(payload.email);

  invite::create(
    &state.pool,
    &state.authz_service,
    &state.email_service,
    authn.id,
    email,
    payload.role,
  )
  .await?;

  Ok(())
}

#[utoipa::path(
  get,
  path = "/api/invites",
  responses(
    (status = StatusCode::OK, description = "List of invites", body = [InviteResponse]),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
    (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
  ),
  security(
    ("session_cookie" = [])
  )
)]
#[axum::debug_handler]
pub async fn get_invites(
  State(state): State<AppState>,
  authn: Authn,
) -> AppResult<Json<Vec<InviteResponse>>> {
  let invites = invite::list_all(&state.pool, &state.authz_service, authn.id).await?;
  let response = invites
    .into_iter()
    .map(InviteResponse::from)
    .collect::<Vec<InviteResponse>>();

  Ok(Json(response))
}

#[utoipa::path(
  post,
  path = "/api/invites/{token}/accept",
  request_body = AcceptInviteRequest,
  params(
    ("token" = String, Path, description = "Invite token")
  ),
  responses(
    (status = StatusCode::OK, description = "Invite accepted successfully"),
    (status = StatusCode::BAD_REQUEST, description = "Validation error or expired invite", body = ErrorResponse),
    (status = StatusCode::NOT_FOUND, description = "Invite not found", body = ErrorResponse),
  ),
)]
pub async fn accept_invite(
  State(state): State<AppState>,
  Path(token): Path<String>,
  ValidatedJson(payload): ValidatedJson<AcceptInviteRequest>,
) -> AppResult<()> {
  invite::accept(
    &state.pool,
    &token,
    RawPassword::new(payload.password),
    payload.first_name,
    payload.last_name,
  )
  .await?;

  Ok(())
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_invite))
    .route("/", get(get_invites))
    .route("/:token/accept", post(accept_invite))
}
