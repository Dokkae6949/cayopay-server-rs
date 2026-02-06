//! List Users Feature
//!
//! Business capability: View all users (admin feature)
//! Uses shared AuthContext to avoid repeated session/auth checks

use axum::{http::StatusCode, response::IntoResponse, Json, Router, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::AuthContext;
use domain::{Permission, Role};

// ===== DTOs =====

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
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
                tracing::error!("DB error in list_users: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal error"}))).into_response()
            }
        }
    }
}

// ===== DB queries =====

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_all_users(pool: &PgPool) -> Result<Vec<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, email, first_name, last_name, role, created_at FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

// ===== Handler =====

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = [UserResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("session_cookie" = []))
)]
pub async fn handle(
    auth: AuthContext,
) -> Result<Json<Vec<UserResponse>>, Error> {
    // Check permissions
    auth.require(Permission::ReadUserDetails)?;
    
    // Get users
    let users = list_all_users(&auth.pool).await?;
    
    let response = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id.to_string(),
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            role: Role::from(u.role),
            created_at: u.created_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/", get(handle))
}
