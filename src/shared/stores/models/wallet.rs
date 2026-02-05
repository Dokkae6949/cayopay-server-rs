use chrono::{DateTime, Utc};
use domain::{wallet::WalletLabel, ActorId, Wallet};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct WalletRow {
  pub id: Uuid,
  pub owner_actor_id: Option<Uuid>,
  pub label: Option<String>,
  pub allow_overdraft: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct WalletCreation {
  pub owner: Option<ActorId>,
  pub label: Option<WalletLabel>,
  pub allow_overdraft: bool,
}

#[derive(Clone)]
pub struct WalletUpdate {
  pub label: Option<Option<WalletLabel>>,
  pub allow_overdraft: Option<bool>,
}

impl From<WalletRow> for Wallet {
  fn from(value: WalletRow) -> Self {
    Self {
      id: value.id.into(),
      owner: value.owner_actor_id.map(Into::into),
      label: value.label.map(|l| l.as_str().into()),
      allow_overdraft: value.allow_overdraft,
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
