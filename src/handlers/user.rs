use axum::{extract::State, routing::get, Json, Router};

use crate::{
  error::AppResult,
  extractors::Auth,
  models::permission::Permission,
  response::UserResponse,
  state::AppState,
  stores::UserStore,
};

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
  auth: Auth,
) -> AppResult<Json<Vec<UserResponse>>> {
  auth.require(Permission::ReadUser)?;
  let users = UserStore::list_all(&state.pool).await?;
  Ok(Json(users.into_iter().map(Into::into).collect()))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_users))
}
