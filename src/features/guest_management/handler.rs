use axum::{extract::State, routing::get, Json, Router};

use crate::shared::error::AppResult;
use crate::shared::extractors::Authz;
use crate::shared::state::AppState;
use domain::Permission;

use super::models::GuestDetailResponse;

/// List all guests
#[utoipa::path(
  get,
  path = "/api/guests",
  responses(
    (status = StatusCode::OK, description = "List of guests", body = [GuestDetailResponse]),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized"),
    (status = StatusCode::FORBIDDEN, description = "Forbidden"),
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn list_guests(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<GuestDetailResponse>>> {
  authz.require(Permission::ReadGuestDetails)?;

  let guests = state.guest_management_service.list_guests().await?;

  Ok(Json(guests))
}

/// Create router for guest management endpoints
pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_guests))
}
