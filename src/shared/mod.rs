//! Shared infrastructure
//!
//! Minimal shared components that don't belong in features.
//! Only includes auth context to avoid DB query duplication.

pub mod auth;

pub use auth::{AuthContext, AuthError};
