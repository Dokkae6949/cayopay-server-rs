use chrono::{DateTime, Utc};

use crate::models::{wallet::WalletId, ActorId};
use crate::types::{Id, Money};

pub type TransactionId = Id<Transaction>;

#[derive(Debug, Clone)]
pub struct Transaction {
  pub id: TransactionId,
  pub source: WalletId,
  pub destination: WalletId,
  pub executor: Option<ActorId>,
  pub amount: Money,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
