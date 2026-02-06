//! Shared Authentication Contexts
//!
//! Provides AuthnContext and AuthzContext extractors to avoid repeated session fetching.
//! These are minimal shared infrastructure that eliminates duplication without adding complexity.

use axum::{async_trait, extract::{FromRequestParts, Request}, http::request::Parts, RequestPartsExt};
use axum_extra::extract::CookieJar;
use sqlx::PgPool;
use std::ops::Deref;
use thiserror::Error;
use uuid::Uuid;

use domain::{Permission, Role, User};

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

// ===== AuthnContext (Authentication) =====

/// Authentication context - validates session and provides user info
/// 
/// Extracts and validates session from cookie, fetches user data.
/// Use this when you need to know WHO the user is.
#[derive(Debug, Clone)]
pub struct AuthnContext {
    pub session_id: String,
    pub user_id: Uuid,
    pub user: User,
    pub pool: PgPool,
}

impl Deref for AuthnContext {
    type Target = User;
    
    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    user_id: Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
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

#[async_trait]
impl FromRequestParts<PgPool> for AuthnContext {
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
        let session: SessionRow = sqlx::query_as(
            "SELECT user_id, expires_at FROM sessions WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?
        .ok_or(AuthError::Unauthorized)?;
        
        // Check expiry
        if session.expires_at < chrono::Utc::now() {
            return Err(AuthError::Unauthorized);
        }
        
        // Fetch user
        let user_row: UserRow = sqlx::query_as(
            "SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at \
             FROM users WHERE id = $1"
        )
        .bind(session.user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AuthError::Unauthorized)?;
        
        Ok(AuthnContext {
            session_id: token.to_string(),
            user_id: session.user_id,
            user: user_row.into(),
            pool: pool.clone(),
        })
    }
}

// ===== AuthzContext (Authorization) =====

/// Authorization context - authentication + permission checking
/// 
/// Includes AuthnContext data plus helper methods for permission checks.
/// Use this when you need to check WHAT the user can do.
#[derive(Debug, Clone)]
pub struct AuthzContext {
    pub authn: AuthnContext,
}

impl Deref for AuthzContext {
    type Target = AuthnContext;
    
    fn deref(&self) -> &Self::Target {
        &self.authn
    }
}

impl AuthzContext {
    /// Check if user has a specific permission
    pub fn require(&self, permission: Permission) -> Result<(), AuthError> {
        if self.authn.user.role.has_permission(permission) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
    
    /// Check if user can assign a specific role
    pub fn can_assign(&self, target_role: Role) -> Result<(), AuthError> {
        if self.authn.user.role.can_assign_role(target_role) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
    
    /// Check if user has any of the given permissions
    pub fn require_any(&self, permissions: &[Permission]) -> Result<(), AuthError> {
        if permissions.iter().any(|p| self.authn.user.role.has_permission(*p)) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
    
    /// Check if user has all of the given permissions
    pub fn require_all(&self, permissions: &[Permission]) -> Result<(), AuthError> {
        if permissions.iter().all(|p| self.authn.user.role.has_permission(*p)) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

#[async_trait]
impl FromRequestParts<PgPool> for AuthzContext {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        pool: &PgPool,
    ) -> Result<Self, Self::Rejection> {
        let authn = AuthnContext::from_request_parts(parts, pool).await?;
        Ok(AuthzContext { authn })
    }
}
