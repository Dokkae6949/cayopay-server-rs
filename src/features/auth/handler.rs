use axum::{
  extract::State,
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};

use crate::shared::error::AppResult;
use crate::shared::extractors::{Authn, ValidatedJson};
use crate::shared::state::AppState;
use domain::{Email, RawPassword};

use super::models::{LoginRequest, UserResponse};
use super::service::AuthService;

/// Login endpoint - authenticates user and returns session cookie
#[utoipa::path(
  post,
  path = "/api/auth/login",
  request_body = LoginRequest,
  responses(
    (status = StatusCode::OK, description = "Login successful", body = UserResponse),
    (status = StatusCode::BAD_REQUEST, description = "Validation error"),
    (status = StatusCode::UNAUTHORIZED, description = "Invalid credentials"),
  )
)]
pub async fn login(
  State(state): State<AppState>,
  jar: CookieJar,
  ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
  let email = Email::new(payload.email);
  let password = RawPassword::new(payload.password);

  let (user, session) = state.auth_service.login(email, password).await?;

  // Create secure HTTP-only cookie with session token
  let cookie = Cookie::build((state.config.session_cookie_name.clone(), session.token))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Strict)
    .expires(cookie::Expiration::DateTime(
      time::OffsetDateTime::now_utc()
        .checked_add(time::Duration::milliseconds(
          session.expires_in.num_milliseconds(),
        ))
        .unwrap(),
    ))
    .build();

  Ok((jar.add(cookie), Json(user.into())))
}

/// Get current authenticated user
#[utoipa::path(
  get,
  path = "/api/auth/me",
  responses(
    (status = StatusCode::OK, description = "Get current user successful", body = UserResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized"),
  ),
  security(
    ("session_cookie" = [])
  )
)]
pub async fn me(Authn(user): Authn) -> AppResult<Json<UserResponse>> {
  Ok(Json(user.into()))
}

/// Create router for auth endpoints
pub fn router() -> Router<AppState> {
  Router::new()
    .route("/login", post(login))
    .route("/me", get(me))
}
