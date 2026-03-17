use crate::{error::AppResult, extractor::Authn, models::GuestResponse};
use application::state::AppState;
use axum::{extract::State, routing::get, Json, Router};

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
  authn: Authn,
) -> AppResult<Json<Vec<GuestResponse>>> {
  let guests = state.guest_service.get_all(authn.id).await?;
  let response: Vec<GuestResponse> = guests.into_iter().map(Into::into).collect();

  Ok(Json(response))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_guests))
}
