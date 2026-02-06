//! Login Feature
//!
//! Business capability: User authentication and session creation
//! Everything in one place: handlers, DB queries, DTOs, errors

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response as AxumResponse}, Json, Router, routing::post};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use domain::{Email, HashedPassword, RawPassword, Role};

// ===== DTOs =====

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Request {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResponseDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
}

// ===== Errors (feature-specific) =====

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid credentials")]
    InvalidCredentials,
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
            Error::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Database(ref e) => {
                tracing::error!("DB error in login: {}", e);
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

// ===== Database queries (inline) =====

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

async fn find_user(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, actor_id, email, password_hash, first_name, last_name, role, created_at, updated_at FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

async fn create_session(pool: &PgPool, user_id: Uuid, token: String, expires: chrono::DateTime<chrono::Utc>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(token)
        .bind(expires)
        .execute(pool)
        .await?;
    Ok(())
}

// ===== Handler =====

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = Request,
    responses(
        (status = 200, description = "Login successful", body = Response),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn handle(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(req): Json<Request>,
) -> Result<(CookieJar, Json<ResponseDto>), Error> {
    // Validate
    req.validate().map_err(|e| Error::Validation(e.to_string()))?;
    
    // Find user
    let user = find_user(&pool, &req.email).await?.ok_or(Error::InvalidCredentials)?;
    
    // Verify password
    let hashed = HashedPassword::new(user.password_hash);
    let raw = RawPassword::new(req.password);
    if !hashed.verify(&raw)? {
        return Err(Error::InvalidCredentials);
    }
    
    // Create session
    let token = Uuid::new_v4().to_string();
    let expires = chrono::Utc::now() + chrono::Duration::days(7);
    create_session(&pool, user.id, token.clone(), expires).await?;
    
    // Cookie
    let cookie = Cookie::build(("cayopay_session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .expires(cookie::Expiration::DateTime(
            time::OffsetDateTime::from_unix_timestamp(expires.timestamp()).unwrap()
        ))
        .build();
    
    Ok((jar.add(cookie), Json(ResponseDto {
        id: user.id.to_string(),
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        role: Role::from(user.role),
    })))
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/login", post(handle))
}
