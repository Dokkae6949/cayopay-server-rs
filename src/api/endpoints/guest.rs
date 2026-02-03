use crate::{
  api::{extractor::Authz, models::GuestResponse},
  domain::{Guest, Permission},
  error::{AppResult, ErrorResponse},
  state::AppState,
  types::Id,
};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{delete, get},
  Json, Router,
};

/// List all guests
#[utoipa::path(
    get,
    context_path = "/api/guests",
    path = "/",
    responses(
        (status = StatusCode::OK, description = "List of all guests", body = Vec<GuestResponse>),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn list_guests(
  State(state): State<AppState>,
  authz: Authz,
) -> AppResult<Json<Vec<GuestResponse>>> {
  authz.require(Permission::ReadGuestDetails)?;

  let guests = state.guest_service.list_all().await?;
  let response: Vec<GuestResponse> = guests.into_iter().map(Into::into).collect();

  Ok(Json(response))
}

/// Remove a guest by ID
#[utoipa::path(
    delete,
    context_path = "/api/guests",
    path = "/{guest_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "Guest removed successfully"),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
        (status = StatusCode::FORBIDDEN, description = "Forbidden", body = ErrorResponse),
        (status = StatusCode::NOT_FOUND, description = "Guest not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn remove_guest(
  State(state): State<AppState>,
  authz: Authz,
  Path(guest_id): Path<Id<Guest>>,
) -> AppResult<StatusCode> {
  authz.require(Permission::RemoveGuest)?;

  state.guest_service.remove_by_id(guest_id).await?;

  Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(list_guests))
    .route("/{guest_id}", delete(remove_guest))
}
