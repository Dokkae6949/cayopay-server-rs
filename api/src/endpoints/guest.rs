use crate::{error::AppResult, extractor::Authz, models::GuestResponse};
use application::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use domain::Permission;

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
  authz.require(Permission::ReadGuestDetails)?;

  let guests = state.guest_service.get_all().await?;
  let response: Vec<GuestResponse> = guests.into_iter().map(Into::into).collect();

  Ok(Json(response))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_guests))
}
