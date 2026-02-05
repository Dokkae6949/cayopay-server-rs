//! List Users Feature
//!
//! Business capability: View all users (admin feature)
//! Everything inline: handler, DB query, auth check, rich DTOs, errors

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json, Router, routing::get};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

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
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Error::Database(ref e) => {
                tracing::error!("DB error in list_users: {}", e);
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
struct CurrentUserRow {
    role: String,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn find_session(pool: &PgPool, token: &str) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE token = $1")
        .bind(token)
        .fetch_optional(pool)
        .await
}

async fn find_user_role(pool: &PgPool, user_id: Uuid) -> Result<Option<CurrentUserRow>, sqlx::Error> {
    sqlx::query_as("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
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
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<Json<Vec<UserResponse>>, Error> {
    // Auth check
    let token = jar.get("cayopay_session").ok_or(Error::Unauthorized)?.value();
    let session = find_session(&pool, token).await?.ok_or(Error::Unauthorized)?;
    if session.expires_at < chrono::Utc::now() {
        return Err(Error::Unauthorized);
    }
    
    // Get user & check permissions
    let user = find_user_role(&pool, session.user_id).await?.ok_or(Error::Unauthorized)?;
    let user_role = Role::from(user.role);
    
    if !user_role.has_permission(Permission::ReadUserDetails) {
        return Err(Error::Forbidden);
    }
    
    // Get users
    let users = list_all_users(&pool).await?;
    
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
