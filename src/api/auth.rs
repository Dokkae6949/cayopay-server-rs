use axum::{
  extract::State,
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
  api::extractor::{Authn, ValidatedJson},
  domain::{Role, User},
  error::AppResult,
  state::AppState,
  types::{Email, Id, RawPassword},
};

#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
  #[validate(email)]
  #[schema(example = "user@example.com")]
  pub email: String,

  #[validate(length(min = 1))]
  #[schema(example = "password123")]
  pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
  pub id: Id<User>,
  pub email: Email,
  pub first_name: String,
  pub last_name: String,
  pub role: Role,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      email: user.email,
      first_name: user.first_name,
      last_name: user.last_name,
      role: user.role,
      created_at: user.created_at,
      updated_at: user.updated_at,
    }
  }
}

#[utoipa::path(
  post,
  context_path = "/api/auth",
  path = "/login",
  request_body = LoginRequest,
  responses(
    (status = StatusCode::OK, description = "Login successful", body = UserResponse),
    (status = StatusCode::BAD_REQUEST, description = "Validation error", body = ErrorResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Invalid credentials", body = ErrorResponse),
    (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn login(
  State(state): State<AppState>,
  jar: CookieJar,
  ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
  let email = Email::new(payload.email);
  let password = RawPassword::new(payload.password);

  let user = state.auth_service.login(email, password).await?;
  let session = state.session_service.create_session(user.id).await?;

  // TODO: Control cookie attributes based on environment (e.g., Secure in production)
  let cookie = Cookie::build((state.config.session_cookie_name.clone(), session.token))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Strict)
    .expires(cookie::Expiration::DateTime(
      time::OffsetDateTime::from_unix_timestamp(session.expires_at.timestamp())
        .expect("timestamp should have been valid"),
    ))
    .build();

  Ok((jar.add(cookie), Json(user.into())))
}

#[utoipa::path(
  get,
  context_path = "/api/auth",
  path = "/me",
  responses(
    (status = StatusCode::OK, description = "Get current user successful", body = UserResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
    (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn me(Authn(user): Authn) -> AppResult<Json<UserResponse>> {
  Ok(Json(user.into()))
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/login", post(login))
    .route("/me", get(me))
}
