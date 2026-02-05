//! Current User Feature
//!
//! Business capability: Get currently authenticated user's information
//! Everything in one place: handler, DB query, auth check, errors

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response as AxumResponse}, Json, Router, routing::get};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

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
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        let (status, msg) = match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Database(ref e) => {
                tracing::error!("DB error in me: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

// ===== DB queries =====

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    user_id: Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    role: String,
}

async fn find_session(pool: &PgPool, token: &str) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE token = $1")
        .bind(token)
        .fetch_optional(pool)
        .await
}

async fn find_user(pool: &PgPool, user_id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, email, first_name, last_name, role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
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
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<Json<ResponseDto>, Error> {
    // Get session from cookie
    let token = jar.get("cayopay_session")
        .ok_or(Error::Unauthorized)?
        .value();
    
    // Find session
    let session = find_session(&pool, token).await?.ok_or(Error::Unauthorized)?;
    
    // Check expiry
    if session.expires_at < chrono::Utc::now() {
        return Err(Error::Unauthorized);
    }
    
    // Get user
    let user = find_user(&pool, session.user_id).await?.ok_or(Error::Unauthorized)?;
    
    Ok(Json(ResponseDto {
        id: user.id.to_string(),
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        role: Role::from(user.role),
    }))
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/me", get(handle))
}
