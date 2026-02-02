use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
  status: String,
}

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
