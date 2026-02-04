use crate::{error::AppResult, extractor::Authz, models::UserResponse};
use application::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use domain::Permission;

/// List all users
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = StatusCode::OK, description = "List of all users", body = Vec<UserResponse>),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
    )
)]
pub async fn list_users(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<UserResponse>>> {
  authz.require(Permission::ReadUserDetails)?;

  let users = state.user_service.get_all().await?;
  let response: Vec<UserResponse> = users.into_iter().map(Into::into).collect();

  Ok(Json(response))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_users))
}
