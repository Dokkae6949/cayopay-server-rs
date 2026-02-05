use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct ActorRow {
  pub id: Uuid,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
