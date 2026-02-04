use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use std::str::FromStr;
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

impl FromStr for Email {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::new(s))
  }
}

impl From<String> for Email {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_debug_impl() {
    let email = Email::new("test@mail.com");
    let debug_str = format!("{:?}", email);
    assert_eq!(debug_str, "Email(***)");
  }
}
