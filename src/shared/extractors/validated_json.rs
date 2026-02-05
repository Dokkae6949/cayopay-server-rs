use axum::{
  async_trait,
  extract::{FromRequest, FromRequestParts, Request},
  http::request::Parts,
  Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::shared::error::{ApiError, AppError};
use crate::shared::state::AppState;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T> FromRequestParts<AppState> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
{
  type Rejection = ApiError;

  async fn from_request_parts(
    _parts: &mut Parts,
    _state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    // This won't work because we need the body, so we'll fail here
    // The caller should use FromRequest instead
    Err(
      AppError::BadRequest("ValidatedJson requires FromRequest, not FromRequestParts".to_string())
        .into(),
    )
  }
}

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ApiError;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(|e| AppError::BadRequest(e.to_string()))?;
    value
      .validate()
      .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(ValidatedJson(value))
  }
}
