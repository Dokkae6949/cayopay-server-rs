//! Accept Invite Feature
//!
//! Business capability: Accept invitation and register new user
//! Everything inline: handler, DB queries, user creation, errors

use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response as AxumResponse}, Json, Router, routing::post};
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use domain::{HashedPassword, RawPassword};

// ===== DTOs =====

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Request {
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1))]
    pub first_name: String,
    #[validate(length(min = 1))]
    pub last_name: String,
}

// ===== Errors =====

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invite not found")]
    NotFound,
    #[error("Invite expired")]
    Expired,
    #[error("User already exists")]
    UserExists,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Password error")]
    Password(#[from] argon2::password_hash::Error),
    #[error("Validation: {0}")]
    Validation(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        let (status, msg) = match self {
            Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Error::Expired => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::UserExists => (StatusCode::CONFLICT, self.to_string()),
            Error::Database(ref e) => {
                tracing::error!("DB error in accept_invite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
            Error::Password(ref e) => {
                tracing::error!("Password error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
            Error::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

// ===== DB queries =====

#[derive(Debug, sqlx::FromRow)]
struct InviteRow {
    id: Uuid,
    email: String,
    role: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

async fn find_invite(pool: &PgPool, token: &str) -> Result<Option<InviteRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, email, role, expires_at FROM invites WHERE token = $1")
        .bind(token)
        .fetch_optional(pool)
        .await
}

async fn user_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

async fn delete_invite(pool: &PgPool, invite_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invites WHERE id = $1")
        .bind(invite_id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn create_user_with_wallet(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    first_name: &str,
    last_name: &str,
    role: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Create actor
    let actor_id: Uuid = sqlx::query_scalar("INSERT INTO actors DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;
    
    // Create user
    sqlx::query("INSERT INTO users (actor_id, email, password_hash, first_name, last_name, role) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(actor_id)
        .bind(email)
        .bind(password_hash)
        .bind(first_name)
        .bind(last_name)
        .bind(role)
        .execute(&mut *tx)
        .await?;
    
    // Create wallet
    sqlx::query("INSERT INTO wallets (owner_actor_id, allow_overdraft) VALUES ($1, false)")
        .bind(actor_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}

// ===== Handler =====

#[utoipa::path(
    post,
    path = "/api/invites/{token}/accept",
    request_body = Request,
    params(("token" = String, Path, description = "Invite token")),
    responses(
        (status = 200, description = "Invite accepted"),
        (status = 400, description = "Validation or expired"),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn handle(
    State(pool): State<PgPool>,
    Path(token): Path<String>,
    Json(req): Json<Request>,
) -> Result<StatusCode, Error> {
    // Validate
    req.validate().map_err(|e| Error::Validation(e.to_string()))?;
    
    // Find invite
    let invite = find_invite(&pool, &token).await?.ok_or(Error::NotFound)?;
    
    // Check expiry
    if invite.expires_at < chrono::Utc::now() {
        return Err(Error::Expired);
    }
    
    // Check if user exists
    if user_exists(&pool, &invite.email).await? {
        return Err(Error::UserExists);
    }
    
    // Hash password
    let raw = RawPassword::new(req.password);
    let hashed = raw.hash()?;
    
    // Create user with wallet
    create_user_with_wallet(
        &pool,
        &invite.email,
        hashed.expose(),
        &req.first_name,
        &req.last_name,
        &invite.role,
    ).await?;
    
    // Delete invite
    delete_invite(&pool, invite.id).await?;
    
    Ok(StatusCode::OK)
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/:token/accept", post(handle))
}
