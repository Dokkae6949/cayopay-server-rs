use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, ToSchema)]
#[serde(transparent)]
#[sqlx(transparent)]
#[schema(example = "user@example.com")]
pub struct Email(String);

impl Email {
  pub fn new(email: impl Into<String>) -> Self {
    Self(email.into())
  }

  pub fn expose(&self) -> &str {
    &self.0
  }
}

impl fmt::Debug for Email {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Email(***)")
  }
}

impl From<String> for Email {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}
