//! Current User Feature
//!
//! Business capability: Get currently authenticated user's information
//! Uses shared AuthContext to avoid repeated session fetching

use axum::{http::StatusCode, response::IntoResponse, Json, Router, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;

use crate::shared::AuthContext;
use domain::Role;

// ===== DTOs =====

#[derive(Debug, Serialize, ToSchema)]
pub struct ResponseDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
}

// ===== Errors =====

#[derive(Debug, Error)]
pub enum Error {
    #[error("Auth error")]
    Auth(#[from] crate::shared::AuthError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Auth(e) => e.into_response(),
        }
    }
}

// ===== Handler =====

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user", body = ResponseDto),
        (status = 401, description = "Unauthorized"),
    ),
    security(("session_cookie" = []))
)]
pub async fn handle(
    auth: AuthContext,
) -> Result<Json<ResponseDto>, Error> {
    // User info is already fetched by AuthContext!
    Ok(Json(ResponseDto {
        id: auth.user.id.to_string(),
        email: auth.user.email.expose().to_string(),
        first_name: auth.user.first_name.clone(),
        last_name: auth.user.last_name.clone(),
        role: auth.user.role,
    }))
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/me", get(handle))
}
