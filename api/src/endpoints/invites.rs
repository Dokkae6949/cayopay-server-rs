use crate::{
  error::AppResult,
  extractor::{Auth, ValidatedJson},
  models::{AcceptInviteRequest, InviteRequest, InviteResponse},
};
use application::{error::AppError, services::auth, state::AppState};
use axum::{
  extract::{Path, State},
  routing::{get, post},
  Json, Router,
};
use chrono::Duration;
use domain::{models::permission::Permission, Email, RawPassword};
use infra::stores::{
  models::{InviteCreation, UserRoleCreation},
  InviteStore, RoleStore, UserRoleStore, UserStore,
};
use uuid::Uuid;

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
  auth: Auth,
  ValidatedJson(payload): ValidatedJson<InviteRequest>,
) -> AppResult<()> {
  auth.require(Permission::SendInvite)?;

  if RoleStore::find_by_name(&state.pool, &payload.role).await.map_err(AppError::from)?.is_none() {
    return Err(AppError::BadRequest(format!("Role '{}' does not exist", payload.role)).into());
  }

  let email = Email::new(payload.email);

  if let Some(invite) = InviteStore::find_by_email(&state.pool, &email).await.map_err(AppError::from)? {
    if invite.is_expired() {
      InviteStore::delete_by_id(&state.pool, &invite.id).await.map_err(AppError::from)?;
    } else {
      return Err(AppError::InviteAlreadySent.into());
    }
  }

  let inviter_name = UserStore::find_by_id(&state.pool, &auth.id)
    .await
    .map_err(AppError::from)?
    .map(|u| format!("{} {}", u.first_name, u.last_name))
    .ok_or(AppError::InvitorMissing(auth.id))?;

  let token = Uuid::new_v4().to_string();

  InviteStore::create(
    &state.pool,
    &InviteCreation {
      invitor: auth.id,
      email: email.clone(),
      token: token.clone(),
      role: payload.role,
      expires_in: Duration::days(7),
    },
  )
  .await
  .map_err(AppError::from)?;

  state.email_service.send_invite(&email, &token, &inviter_name).await.map_err(AppError::from)?;

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
  auth: Auth,
) -> AppResult<Json<Vec<InviteResponse>>> {
  auth.require(Permission::ViewInvite)?;
  let invites = InviteStore::list_all(&state.pool).await.map_err(AppError::from)?;
  Ok(Json(invites.into_iter().map(InviteResponse::from).collect()))
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
  let invite = InviteStore::find_by_token(&state.pool, &token)
    .await
    .map_err(AppError::from)?
    .ok_or(AppError::NotFound)?;

  if invite.is_expired() {
    return Err(AppError::InviteExpired.into());
  }

  let user = auth::register(
    &state.pool,
    invite.email.clone(),
    RawPassword::new(payload.password),
    payload.first_name,
    payload.last_name,
  )
  .await?;

  let role = RoleStore::find_by_name(&state.pool, &invite.role)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::BadRequest(format!(
      "The role '{}' no longer exists; the invite must be re-issued",
      invite.role
    )))?;

  UserRoleStore::assign(
    &state.pool,
    &UserRoleCreation {
      user_id: user.id,
      role_id: role.id,
    },
  )
  .await
  .map_err(AppError::from)?;

  InviteStore::delete_by_id(&state.pool, &invite.id).await.map_err(AppError::from)?;

  Ok(())
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_invite))
    .route("/", get(get_invites))
    .route("/:token/accept", post(accept_invite))
}
