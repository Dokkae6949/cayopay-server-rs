//! List Invites Feature
//!
//! Business capability: View all pending invitations (admin feature)
//! Everything inline: handler, DB query, auth check, rich DTOs, errors

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response as AxumResponse}, Json, Router, routing::get};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use domain::{Permission, Role};

// ===== DTOs (rich data) =====

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    pub id: String,
    pub email: String,
    pub role: Role,
    pub invitor_name: String, // Rich: include name not just ID
    pub expires_at: chrono::DateTime<chrono::Utc>,
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
    fn into_response(self) -> AxumResponse {
        let (status, msg) = match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Error::Database(ref e) => {
                tracing::error!("DB error in list_invites: {}", e);
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
    role: String,
}

#[derive(Debug, sqlx::FromRow)]
struct InviteRow {
    id: Uuid,
    email: String,
    role: String,
    invitor_first_name: String,
    invitor_last_name: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn find_session(pool: &PgPool, token: &str) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE token = $1")
        .bind(token)
        .fetch_optional(pool)
        .await
}

async fn find_user(pool: &PgPool, user_id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

async fn list_invites(pool: &PgPool) -> Result<Vec<InviteRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT i.id, i.email, i.role, u.first_name as invitor_first_name, u.last_name as invitor_last_name, \
         i.expires_at, i.created_at \
         FROM invites i \
         JOIN users u ON i.invitor_user_id = u.id \
         ORDER BY i.created_at DESC"
    )
    .fetch_all(pool)
    .await
}

// ===== Handler =====

#[utoipa::path(
    get,
    path = "/api/invites",
    responses(
        (status = 200, description = "List of invites", body = [InviteResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("session_cookie" = []))
)]
pub async fn handle(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<Json<Vec<InviteResponse>>, Error> {
    // Auth check
    let token = jar.get("cayopay_session").ok_or(Error::Unauthorized)?.value();
    let session = find_session(&pool, token).await?.ok_or(Error::Unauthorized)?;
    if session.expires_at < chrono::Utc::now() {
        return Err(Error::Unauthorized);
    }
    
    // Get user & check permissions
    let user = find_user(&pool, session.user_id).await?.ok_or(Error::Unauthorized)?;
    let user_role = Role::from(user.role);
    
    if !user_role.has_permission(Permission::ViewInvite) {
        return Err(Error::Forbidden);
    }
    
    // Get invites with rich data
    let invites = list_invites(&pool).await?;
    
    let response = invites
        .into_iter()
        .map(|inv| InviteResponse {
            id: inv.id.to_string(),
            email: inv.email,
            role: Role::from(inv.role),
            invitor_name: format!("{} {}", inv.invitor_first_name, inv.invitor_last_name),
            expires_at: inv.expires_at,
            created_at: inv.created_at,
        })
        .collect();
    
    Ok(Json(response))
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/", get(handle))
}
