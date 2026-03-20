use axum::{
  extract::{Path, State},
  routing::{get, post},
  Json, Router,
};
use chrono::Duration;
use uuid::Uuid;

use crate::{
  error::{AppError, AppResult},
  extractors::{Authz, ValidatedJson},
  models::permission::GlobalPermission,
  response::{AcceptInviteRequest, InviteRequest, InviteResponse},
  services::{auth, PermissionEngine},
  state::AppState,
  stores::{
    models::{InviteCreation, UserRoleCreation},
    InviteStore, RoleStore, UserRoleStore, UserStore,
  },
  types::{Email, RawPassword},
};

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
  authz: Authz,
  ValidatedJson(payload): ValidatedJson<InviteRequest>,
) -> AppResult<()> {
  authz.global().require(GlobalPermission::SendInvite).await?;

  let mut conn = state.pool.acquire().await?;

  if RoleStore::find_by_name(&mut *conn, &payload.role)
    .await?
    .is_none()
  {
    return Err(AppError::BadRequest(format!(
      "Role '{}' does not exist",
      payload.role
    )));
  }

  let email = Email::new(payload.email);

  if let Some(invite) = InviteStore::find_by_email(&mut *conn, &email).await? {
    if invite.is_expired() {
      InviteStore::delete_by_id(&mut *conn, &invite.id).await?;
    } else {
      return Err(AppError::InviteAlreadySent);
    }
  }

  let inviter_name = UserStore::find_by_id(&mut *conn, &authz.id)
    .await?
    .map(|u| format!("{} {}", u.first_name, u.last_name))
    .unwrap_or_else(|| "Someone".to_string());

  let token = Uuid::new_v4().to_string();

  InviteStore::create(
    &mut *conn,
    &InviteCreation {
      invitor: authz.id,
      email: email.clone(),
      token: token.clone(),
      role: payload.role,
      expires_in: Duration::days(7),
    },
  )
  .await?;

  state
    .email_service
    .send_invite(&email, &token, &inviter_name)
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
  authz: Authz,
) -> AppResult<Json<Vec<InviteResponse>>> {
  authz.global().require(GlobalPermission::ViewInvite).await?;

  let mut conn = state.pool.acquire().await?;
  let invites = InviteStore::list_all(&mut *conn).await?;

  Ok(Json(
    invites.into_iter().map(InviteResponse::from).collect(),
  ))
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
  let invite = {
    let mut conn = state.pool.acquire().await?;
    InviteStore::find_by_token(&mut *conn, &token)
      .await?
      .ok_or(AppError::NotFound)?
  };

  if invite.is_expired() {
    return Err(AppError::InviteExpired);
  }

  let user = auth::register(
    &state.pool,
    invite.email.clone(),
    RawPassword::new(payload.password),
    payload.first_name,
    payload.last_name,
  )
  .await?;

  let mut conn = state.pool.acquire().await?;

  let role = RoleStore::find_by_name(&mut *conn, &invite.role)
    .await?
    .ok_or_else(|| {
      AppError::BadRequest(format!(
        "Invite role '{}' does not exist; ask an admin to fix this invite",
        invite.role
      ))
    })?;

  UserRoleStore::assign(
    &mut *conn,
    &UserRoleCreation {
      user_id: user.id,
      role_id: role.id,
    },
  )
  .await?;

  InviteStore::delete_by_id(&mut *conn, &invite.id).await?;

  Ok(())
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_invite))
    .route("/", get(get_invites))
    .route("/:token/accept", post(accept_invite))
}
