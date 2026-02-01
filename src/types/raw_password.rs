use argon2::{
  password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
  Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use utoipa::ToSchema;

use crate::types::HashedPassword;

#[derive(Clone, Serialize, Deserialize, Type, ToSchema)]
#[serde(transparent)]
#[sqlx(transparent)]
#[schema(example = "password123")]
pub struct RawPassword(String);

impl RawPassword {
  pub fn new(password: impl Into<String>) -> Self {
    Self(password.into())
  }

  pub fn expose(&self) -> &str {
    &self.0
  }

  pub fn hash(&self) -> Result<HashedPassword, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(self.0.as_bytes(), &salt)?.to_string();
    Ok(HashedPassword::new(password_hash))
  }
}

impl fmt::Debug for RawPassword {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "RawPassword(***)")
  }
}

impl From<String> for RawPassword {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}
