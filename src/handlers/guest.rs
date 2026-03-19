use axum::{extract::State, routing::get, Json, Router};

use crate::{
  error::AppResult,
  extractors::Authz,
  models::permission::READ_GUEST,
  response::GuestResponse,
  state::AppState,
  stores::GuestStore,
};

#[utoipa::path(
    get,
    path = "/api/guests",
    responses(
        (status = StatusCode::OK, description = "List of all guests", body = Vec<GuestResponse>),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
    )
)]
pub async fn list_guests(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<GuestResponse>>> {
  authz.require_global(READ_GUEST).await?;
  let guests = GuestStore::list_all(&state.pool).await?;
  Ok(Json(guests.into_iter().map(Into::into).collect()))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_guests))
}
