//! Shared Authentication Context
//!
//! Provides AuthContext extractor to avoid repeated session fetching.
//! This is minimal shared infrastructure that eliminates duplication without adding complexity.

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use std::ops::Deref;
use thiserror::Error;
use uuid::Uuid;

use domain::{Id, Permission, Role, Session, User};

// ===== Errors =====

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::Json;
        
        let (status, msg) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AuthError::Database(ref e) => {
                tracing::error!("Auth DB error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

// ===== AuthContext (Authentication & Authorization) =====

/// Authentication & Authorization context
/// 
/// Validates session and provides user info + permission checking helpers.
/// Combines both authentication (WHO is the user) and authorization (WHAT can they do).
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub session: Session,
    pub user: User,
    pub pool: PgPool,
}

impl Deref for AuthContext {
    type Target = User;
    
    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

impl AuthContext {
    /// Check if user has a specific permission
    pub fn require(&self, permission: Permission) -> Result<(), AuthError> {
        if self.user.role.has_permission(permission) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
    
    /// Check if user can assign a specific role
    pub fn can_assign(&self, target_role: Role) -> Result<(), AuthError> {
        if self.user.role.can_assign_role(target_role) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
    
    /// Check if user has any of the given permissions
    pub fn require_any(&self, permissions: &[Permission]) -> Result<(), AuthError> {
        if permissions.iter().any(|p| self.user.role.has_permission(*p)) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
    
    /// Check if user has all of the given permissions
    pub fn require_all(&self, permissions: &[Permission]) -> Result<(), AuthError> {
        if permissions.iter().all(|p| self.user.role.has_permission(*p)) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    token: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    expires_in_seconds: i64,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    actor_id: Uuid,
    email: String,
    password_hash: String,
    first_name: String,
    last_name: String,
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: domain::Id::from(row.id),
            actor_id: domain::Id::from(row.actor_id),
            email: domain::Email::new(row.email),
            password: domain::HashedPassword::new(row.password_hash),
            first_name: row.first_name,
            last_name: row.last_name,
            role: Role::from(row.role),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Session {
            id: Id::from(row.id),
            user_id: Id::from(row.user_id),
            token: row.token,
            user_agent: row.user_agent,
            ip_address: row.ip_address,
            expires_in: Duration::seconds(row.expires_in_seconds),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl FromRequestParts<PgPool> for AuthContext {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        pool: &PgPool,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookie jar
        let jar = parts
            .extract::<CookieJar>()
            .await
            .map_err(|_| AuthError::Unauthorized)?;
        
        // Get session token from cookie
        let token = jar
            .get("cayopay_session")
            .ok_or(AuthError::Unauthorized)?
            .value();
        
        // Fetch session
        let session_row: SessionRow = sqlx::query_as(
            "SELECT id, user_id, token, user_agent, ip_address, \
             EXTRACT(EPOCH FROM (expires_at - created_at))::bigint as expires_in_seconds, \
             created_at, updated_at \
             FROM sessions WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?
        .ok_or(AuthError::Unauthorized)?;
        
        let session: Session = session_row.into();
        
        // Check expiry
        if session.is_expired() {
            return Err(AuthError::Unauthorized);
        }
        
        // Fetch user
        let user_row: UserRow = sqlx::query_as(
            "SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at \
             FROM users WHERE id = $1"
        )
        .bind(session.user_id.into_inner())
        .fetch_optional(pool)
        .await?
        .ok_or(AuthError::Unauthorized)?;
        
        Ok(AuthContext {
            session,
            user: user_row.into(),
            pool: pool.clone(),
        })
    }
}
