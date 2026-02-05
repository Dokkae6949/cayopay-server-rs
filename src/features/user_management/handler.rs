use axum::{http::StatusCode, routing::get, Json, Router};

use crate::shared::error::AppResult;
use crate::shared::extractors::Authz;
use crate::shared::state::AppState;
use domain::Permission;

use super::models::UserDetailResponse;

/// List all users
#[utoipa::path(
  get,
  path = "/api/users",
  responses(
    (status = StatusCode::OK, description = "List of users", body = [UserDetailResponse]),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized"),
    (status = StatusCode::FORBIDDEN, description = "Forbidden"),
  ),
  security(
    ("session_cookie" = [])
  )
)]
#[axum::debug_handler]
pub async fn list_users(authz: Authz) -> AppResult<Json<Vec<UserDetailResponse>>> {
  authz.require(Permission::ReadUserDetails)?;

  let users = authz.state.user_management_service.list_users().await?;

  Ok(Json(users))
}

/// Create router for user management endpoints
pub fn router() -> Router<AppState> {
  Router::<AppState>::new().route("/", get(list_users))
}
