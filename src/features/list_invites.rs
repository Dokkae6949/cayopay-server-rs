//! List Invites Feature
//!
//! Business capability: View all pending invitations (admin feature)
//! Uses shared AuthContext to avoid repeated session/auth checks

use axum::{http::StatusCode, response::IntoResponse, Json, Router, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::AuthContext;
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
                tracing::error!("DB error in list_invites: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal error"}))).into_response()
            }
        }
    }
}

// ===== DB queries =====

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
    auth: AuthContext,
) -> Result<Json<Vec<InviteResponse>>, Error> {
    // Check permissions (AuthContext already has user loaded!)
    auth.require(Permission::ViewInvite)?;
    
    // Get invites with rich data
    let invites = list_invites(&auth.pool).await?;
    
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
