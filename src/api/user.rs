use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{delete, get},
  Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
  api::extractor::Authz,
  domain::{Permission, Role, User},
  error::{AppResult, ErrorResponse},
  state::AppState,
  types::{Email, Id},
};

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
  pub id: Id<User>,
  pub actor_id: Id<crate::domain::Actor>,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      actor_id: user.actor_id,
      email: user.email,
      first_name: user.first_name,
      last_name: user.last_name,
      role: user.role,
      created_at: user.created_at,
      updated_at: user.updated_at,
    }
  }
}

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
