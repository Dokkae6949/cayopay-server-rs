use axum::http::StatusCode;
use axum::{
  extract::State,
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};

use crate::{
  api::{
    extractor::{Authn, ValidatedJson},
    models::{LoginRequest, UserResponse},
  },
  error::{AppError, AppResult, ErrorResponse},
  state::AppState,
  types::{Email, RawPassword},
};

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

  let expires_at = time::OffsetDateTime::from_unix_timestamp(session.expires_at.timestamp())
    .map_err(|e| {
      tracing::error!("Failed to convert session expiration timestamp: {}", e);
      AppError::InternalServerError
    })?;

  let cookie = Cookie::build((state.config.session_cookie_name.clone(), session.token))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Strict)
    .expires(cookie::Expiration::DateTime(expires_at))
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
