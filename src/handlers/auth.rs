use axum::{
  extract::State,
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};
use chrono::Duration;
use uuid::Uuid;

use crate::{
  error::AppResult,
  extractors::{Authn, ValidatedJson},
  response::{LoginRequest, UserResponse},
  state::AppState,
  stores::{models::SessionCreation, SessionStore, UserStore},
  types::{Email, RawPassword},
  error::AppError,
};

#[utoipa::path(
  post,
  path = "/api/auth/login",
  request_body = LoginRequest,
  responses(
    (status = StatusCode::OK, description = "Login successful", body = UserResponse),
    (status = StatusCode::BAD_REQUEST, description = "Validation error", body = ErrorResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Invalid credentials", body = ErrorResponse),
  )
)]
pub async fn login(
  State(state): State<AppState>,
  jar: CookieJar,
  ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
  let email = Email::new(payload.email);
  let password = RawPassword::new(payload.password);

  let user = UserStore::find_by_email(&state.pool, &email)
    .await?
    .ok_or(AppError::Authentication)?;

  if !user.password.verify(&password)? {
    return Err(AppError::Authentication);
  }

  let session = SessionStore::create(
    &state.pool,
    &SessionCreation {
      user_id: user.id,
      token: Uuid::new_v4().to_string(),
      user_agent: None,
      ip_address: None,
      expires_in: Duration::days(state.config.session_expiration_days),
    },
  )
  .await?;

  // TODO: Control cookie attributes based on environment (e.g., Secure in production)
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

#[utoipa::path(
  get,
  path = "/api/auth/me",
  responses(
    (status = StatusCode::OK, description = "Get current user successful", body = UserResponse),
    (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = ErrorResponse),
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
