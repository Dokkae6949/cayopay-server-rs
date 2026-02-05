//! Send Invite Feature
//!
//! Business capability: Owner/Admin sends invitation to new user
//! Everything inline: handler, DB queries, email sending, auth check, errors

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json, Router, routing::post};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use domain::{Email, Permission, Role};

// ===== DTOs =====

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Request {
    #[validate(email)]
    pub email: String,
    pub role: Role,
}

// ===== Errors =====

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Invite already sent")]
    AlreadySent,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Email error: {0}")]
    Email(String),
    #[error("Validation: {0}")]
    Validation(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Error::AlreadySent => (StatusCode::CONFLICT, self.to_string()),
            Error::Database(ref e) => {
                tracing::error!("DB error in send_invite: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
            Error::Email(ref msg) => {
                tracing::error!("Email error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
            Error::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
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
    role: String,
    first_name: String,
    last_name: String,
}

async fn find_session(pool: &PgPool, token: &str) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE token = $1")
        .bind(token)
        .fetch_optional(pool)
        .await
}

async fn find_user(pool: &PgPool, user_id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as("SELECT id, role, first_name, last_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

async fn find_invite_by_email(pool: &PgPool, email: &str) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM invites WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

async fn create_invite(
    pool: &PgPool,
    invitor_id: Uuid,
    email: &str,
    token: &str,
    role: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO invites (invitor_user_id, email, token, role, expires_at) VALUES ($1, $2, $3, $4, $5)")
        .bind(invitor_id)
        .bind(email)
        .bind(token)
        .bind(role)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

// ===== Email (generic mail object approach) =====

#[derive(Debug)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

async fn send_email(mail: Mail, smtp_config: &SmtpConfig) -> Result<(), String> {
    use lettre::{Message, SmtpTransport, Transport, message::header::ContentType};
    use lettre::transport::smtp::authentication::Credentials;
    
    let email = Message::builder()
        .from(smtp_config.from.parse().map_err(|e| format!("Invalid from: {}", e))?)
        .to(mail.to.parse().map_err(|e| format!("Invalid to: {}", e))?)
        .subject(mail.subject)
        .header(ContentType::TEXT_HTML)
        .body(mail.body)
        .map_err(|e| format!("Build error: {}", e))?;
    
    let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());
    let mailer = SmtpTransport::relay(&smtp_config.host)
        .map_err(|e| format!("SMTP error: {}", e))?
        .credentials(creds)
        .port(smtp_config.port)
        .build();
    
    mailer.send(&email).map_err(|e| format!("Send error: {}", e))?;
    Ok(())
}

#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

// ===== Handler =====

#[utoipa::path(
    post,
    path = "/api/invites",
    request_body = Request,
    responses(
        (status = 200, description = "Invite sent"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("session_cookie" = []))
)]
pub async fn handle(
    State((pool, smtp)): State<(PgPool, SmtpConfig)>,
    jar: CookieJar,
    Json(req): Json<Request>,
) -> Result<StatusCode, Error> {
    // Validate
    req.validate().map_err(|e| Error::Validation(e.to_string()))?;
    
    // Auth check
    let token = jar.get("cayopay_session").ok_or(Error::Unauthorized)?.value();
    let session = find_session(&pool, token).await?.ok_or(Error::Unauthorized)?;
    if session.expires_at < chrono::Utc::now() {
        return Err(Error::Unauthorized);
    }
    
    // Get user & check permissions
    let user = find_user(&pool, session.user_id).await?.ok_or(Error::Unauthorized)?;
    let user_role = Role::from(user.role.clone());
    
    if !user_role.has_permission(Permission::SendInvite) {
        return Err(Error::Forbidden);
    }
    
    if !user_role.can_assign_role(req.role) {
        return Err(Error::Forbidden);
    }
    
    // Check if invite exists
    if find_invite_by_email(&pool, &req.email).await?.is_some() {
        return Err(Error::AlreadySent);
    }
    
    // Create invite
    let invite_token = Uuid::new_v4().to_string();
    let expires = chrono::Utc::now() + chrono::Duration::days(7);
    create_invite(&pool, user.id, &req.email, &invite_token, &req.role.to_string(), expires).await?;
    
    // Send email
    let inviter_name = format!("{} {}", user.first_name, user.last_name);
    let mail = Mail {
        to: req.email,
        subject: "You're invited to CayoPay".to_string(),
        body: format!(
            "<h1>CayoPay Invitation</h1><p>You have been invited by <b>{}</b>.</p><p>Token: <code>{}</code></p>",
            inviter_name, invite_token
        ),
    };
    
    send_email(mail, &smtp).await.map_err(Error::Email)?;
    
    Ok(StatusCode::OK)
}

pub fn router() -> Router<(PgPool, SmtpConfig)> {
    Router::new().route("/", post(handle))
}
