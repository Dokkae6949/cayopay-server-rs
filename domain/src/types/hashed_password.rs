use argon2::{
  password_hash::{PasswordHash, PasswordVerifier},
  Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use utoipa::ToSchema;

use crate::types::RawPassword;

#[derive(Clone, Default, Serialize, Deserialize, Type, ToSchema)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct HashedPassword(String);

impl HashedPassword {
  pub fn new(hash: impl Into<String>) -> Self {
    Self(hash.into())
  }

  pub fn expose(&self) -> &str {
    &self.0
  }

  pub fn verify(&self, password: &RawPassword) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(&self.0)?;
    Ok(
      Argon2::default()
        .verify_password(password.expose().as_bytes(), &parsed_hash)
        .is_ok(),
    )
  }
}

impl fmt::Debug for HashedPassword {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "HashedPassword(***)")
  }
}

impl From<String> for HashedPassword {
  fn from(value: String) -> Self {
    Self::new(value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_password_verification() {
    let password = RawPassword::new("password123");
    let hashed = password.hash().expect("failed to hash password");

    assert!(hashed.verify(&password).expect("failed to verify password"));

    let wrong_password = RawPassword::new("wrongpassword");
    assert!(!hashed
      .verify(&wrong_password)
      .expect("failed to verify password"));
  }

  #[test]
  fn test_debug_impl() {
    let hashed = HashedPassword::new("somehash");
    assert_eq!(format!("{:?}", hashed), "HashedPassword(***)");
  }
}
