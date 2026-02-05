use axum::{async_trait, extract::FromRequest, http::Request, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::{ApiError, AppError};

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ApiError;

  async fn from_request(
    req: Request<axum::body::Body>,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(|e| AppError::BadRequest(e.to_string()))?;
    value
      .validate()
      .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(ValidatedJson(value))
  }
}
