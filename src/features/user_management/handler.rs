use axum::{extract::State, routing::get, Json, Router};

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
pub async fn list_users(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<UserDetailResponse>>> {
  authz.require(Permission::ReadUserDetails)?;

  let users = state.user_management_service.list_users().await?;

  Ok(Json(users))
}

/// Create router for user management endpoints
pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_users))
}
