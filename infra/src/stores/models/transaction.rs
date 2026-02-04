use chrono::{DateTime, Utc};
use domain::{types::Money, wallet::WalletId, ActorId, Transaction};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct TransactionRow {
  pub id: Uuid,
  pub source_wallet_id: Uuid,
  pub destination_wallet_id: Uuid,
  pub executor_actor_id: Option<Uuid>,
  pub amount: i64,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct TransactionCreation {
  pub source: WalletId,
  pub destination: WalletId,
  pub executor: Option<ActorId>,
  pub amount: Money,
  pub description: Option<String>,
}

impl From<TransactionRow> for Transaction {
  fn from(value: TransactionRow) -> Self {
    Self {
      id: value.id.into(),
      source: value.source_wallet_id.into(),
      destination: value.destination_wallet_id.into(),
      executor: value.executor_actor_id.map(Into::into),
      amount: value.amount.into(),
      description: value.description,
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
