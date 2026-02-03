use crate::{
  api::{extractor::Authz, models::UserResponse},
  domain::{Permission, User},
  error::{AppResult, ErrorResponse},
  state::AppState,
  types::Id,
};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{delete, get},
  Json, Router,
};

/// List all users
#[utoipa::path(
    get,
    context_path = "/api/users",
    path = "/",
    responses(
        (status = StatusCode::OK, description = "List of all users", body = Vec<UserResponse>),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn list_users(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<UserResponse>>> {
  authz.require(Permission::ReadUserDetails)?;

  let users = state.actor_service.list_users().await?;
  let response: Vec<UserResponse> = users.into_iter().map(Into::into).collect();

  Ok(Json(response))
}

/// Remove a user by ID
#[utoipa::path(
    delete,
    context_path = "/api/users",
    path = "/{user_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "User removed successfully"),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn remove_user(
  State(state): State<AppState>,
  authz: Authz,
  Path(user_id): Path<Id<User>>,
) -> AppResult<StatusCode> {
  authz.require(Permission::RemoveUser)?;

  state.actor_service.remove_user_by_id(user_id).await?;

  Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(list_users))
    .route("/{user_id}", delete(remove_user))
}
