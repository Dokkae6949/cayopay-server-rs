//! List Guests Feature
//!
//! Business capability: View all guests (admin feature)
//! Uses shared AuthContext to avoid repeated session/auth checks

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, Router, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::AuthContext;
use domain::Permission;

// ===== DTOs =====

#[derive(Debug, Serialize, ToSchema)]
pub struct GuestResponse {
    pub id: String,
    pub email: Option<String>,
    pub verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ===== Errors =====

#[derive(Debug, Error)]
pub enum Error {
    #[error("Auth error")]
    Auth(#[from] crate::shared::AuthError),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Auth(e) => e.into_response(),
            Error::Database(ref e) => {
                tracing::error!("DB error in list_guests: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal error"}))).into_response()
            }
        }
    }
}

// ===== DB queries =====

#[derive(Debug, sqlx::FromRow)]
struct GuestRow {
    id: Uuid,
    email: Option<String>,
    verified: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_all_guests(pool: &PgPool) -> Result<Vec<GuestRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, email, verified, created_at FROM guests ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

// ===== Handler =====

#[utoipa::path(
    get,
    path = "/api/guests",
    responses(
        (status = 200, description = "List of guests", body = [GuestResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("session_cookie" = []))
)]
pub async fn handle(
    auth: AuthContext,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<GuestResponse>>, Error> {
    // Check permissions
    auth.require(Permission::ReadGuestDetails)?;
    
    // Get guests
    let guests = list_all_guests(&pool).await?;
    
    let response = guests
        .into_iter()
        .map(|g| GuestResponse {
            id: g.id.to_string(),
            email: g.email,
            verified: g.verified,
            created_at: g.created_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/", get(handle))
}
