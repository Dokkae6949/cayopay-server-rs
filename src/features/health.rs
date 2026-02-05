//! Health Check Feature
//!
//! Business capability: System health monitoring

use axum::{http::StatusCode, response::IntoResponse, Json, Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ResponseDto {
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Server is healthy", body = ResponseDto)
    )
)]
pub async fn handle() -> impl IntoResponse {
    (StatusCode::OK, Json(ResponseDto {
        status: "ok".to_string(),
    }))
}

pub fn router() -> Router {
    Router::new().route("/health", get(handle))
}
