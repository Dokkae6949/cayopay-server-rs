use axum::{extract::State, routing::get, Json, Router};

use crate::{
  error::AppResult, extractors::Authz, models::permission::GlobalPermission,
  response::GuestResponse, services::PermissionEngine, state::AppState, stores::GuestStore,
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
  authz.global().require(GlobalPermission::ReadGuest).await?;

  let mut conn = state.pool.acquire().await?;
  let guests = GuestStore::list_all(&mut *conn).await?;

  Ok(Json(guests.into_iter().map(Into::into).collect()))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_guests))
}
