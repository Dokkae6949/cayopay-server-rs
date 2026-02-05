use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use super::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
  pub status: String,
}

#[utoipa::path(
  get,
  path = "/api/health",
  responses(
    (status = 200, description = "Server is healthy", body = HealthResponse)
  )
)]
pub async fn health_check() -> impl IntoResponse {
  Json(HealthResponse {
    status: "ok".to_string(),
  })
}

pub fn health_router() -> Router<AppState> {
  Router::new().route("/health", get(health_check))
}
