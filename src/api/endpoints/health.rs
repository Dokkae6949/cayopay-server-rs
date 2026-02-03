use crate::{api::models::HealthResponse, state::AppState};
use axum::{response::IntoResponse, routing::get, Json, Router};

#[utoipa::path(
  get,
  context_path = "/api",
  path = "/health",
  responses(
    (status = 200, description = "Server is healthy", body = HealthResponse)
  )
)]
pub async fn health_check() -> impl IntoResponse {
  Json(HealthResponse {
    status: "ok".to_string(),
  })
}

pub fn router() -> Router<AppState> {
  Router::new().route("/health", get(health_check))
}
