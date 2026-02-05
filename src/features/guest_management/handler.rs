use axum::{http::StatusCode, routing::get, Json, Router};

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
#[axum::debug_handler]
pub async fn list_guests(authz: Authz) -> AppResult<Json<Vec<GuestDetailResponse>>> {
  authz.require(Permission::ReadGuestDetails)?;

  let guests = authz.state.guest_management_service.list_guests().await?;

  Ok(Json(guests))
}

/// Create router for guest management endpoints
pub fn router() -> Router<AppState> {
  Router::<AppState>::new().route("/", get(list_guests))
}
