use chrono::{DateTime, Utc};
use domain::Permission;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct PermissionRow {
  pub id: Uuid,
  pub action: String,
  pub subject: String,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PermissionCreation {
  pub action: String,
  pub subject: String,
  pub description: Option<String>,
}

impl From<PermissionRow> for Permission {
  fn from(value: PermissionRow) -> Self {
    Self {
      id: value.id.into(),
      action: value.action,
      subject: value.subject,
      description: value.description,
      created_at: value.created_at,
    }
  }
}
