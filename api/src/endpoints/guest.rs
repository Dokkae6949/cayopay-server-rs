use crate::{error::AppResult, extractor::Authn, models::GuestResponse};
use application::{error::AppError, state::AppState};
use axum::{extract::State, routing::get, Json, Router};
use domain::models::permission::Permission;
use infra::stores::GuestStore;

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
  state.authz_service.require(authn.id, Permission::ReadGuest).await?;
  let guests = GuestStore::list_all(&state.pool).await.map_err(AppError::from)?;
  Ok(Json(guests.into_iter().map(Into::into).collect()))
}

pub fn router() -> Router<AppState> {
  Router::new().route("/", get(list_guests))
}
